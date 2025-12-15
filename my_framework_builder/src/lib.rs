use std::{env, fs, io, path::PathBuf};

pub fn prepare_runner_and_emit_env() -> io::Result<()> {
    let exe = env::current_exe()?;
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let runner = out_dir.join("my_framework_runner");
    fs::copy(&exe, &runner)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&runner)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&runner, perms)?;
    }
    println!("cargo:rustc-env=MY_FRAMEWORK_RUNNER={}", runner.display());
    Ok(())
}

pub fn run_as_runner() -> std::io::Result<()> {
    use clap::Parser;
    use plugin_api::{PluginEntry, PluginService, inventory};
    use std::sync::Arc;

    #[derive(Parser)]
    struct Args {
        #[arg(long)]
        runner: String,
        #[arg(long)]
        socket: String,
    }

    struct Wrapper(Arc<dyn plugin_api::Plugin>);

    impl PluginService for Wrapper {
        async fn transform(&self, input: String) -> String {
            self.0.transform(&input).expect("plugin transform failed")
        }
    }

    let args = Args::parse();
    let target = args.runner;

    let mut selected_plugin: Option<Arc<dyn plugin_api::Plugin>> = None;

    for entry in inventory::iter::<PluginEntry> {
        if entry.name == target {
            selected_plugin = Some(Arc::from((entry.constructor)()));
            break;
        }
    }

    let plugin = selected_plugin.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("no matching plugin found for '{target}'"),
        )
    })?;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    rt.block_on(async move {
        // Remove socket if it exists
        let _ = tokio::fs::remove_file(&args.socket).await;

        let listener = tokio::net::UnixListener::bind(&args.socket)?;

        loop {
            let (stream, _) = listener.accept().await?;
            let transport = std::sync::Arc::new(rapace::StreamTransport::new(stream));
            let service = Wrapper(plugin.clone());

            let _ = plugin_api::PluginServiceServer::new(service)
                .serve(transport)
                .await;

            // Exit after serving one request since this is a one-shot runner
            break;
        }

        Ok(())
    })
}
