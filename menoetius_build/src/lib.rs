use std::{env, fs, io, path::PathBuf, sync::Arc};

use rapace::RpcError;

pub fn prepare_runner_and_emit_env() -> io::Result<()> {
    let exe = env::current_exe()?;
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let runner = out_dir.join("menoetius_runner");
    fs::copy(&exe, &runner)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&runner)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&runner, perms)?;
    }
    println!("cargo:rustc-env=MENOETIUS_RUNNER={}", runner.display());
    Ok(())
}

/// Run the process in runner mode. This is the runner implementation that
/// listens on a unix socket for one request, dispatches to the selected plugin,
/// and returns.
pub async fn run_runner_mode<T>() -> Result<
    (
        T,
        Arc<
            rapace::StreamTransport<
                tokio::io::ReadHalf<tokio::net::UnixStream>,
                tokio::io::WriteHalf<tokio::net::UnixStream>,
            >,
        >,
    ),
    std::io::Error,
>
where
    T: Send + Sync + 'static,
{
    use clap::Parser;
    use menoetius::{PluginEntry, inventory};

    #[derive(Parser)]
    struct Args {
        #[arg(long)]
        runner: String,
        #[arg(long)]
        socket: String,
    }

    let args = Args::parse();
    let target = args.runner;

    let mut selected_plugin: Option<T> = None;

    for entry in inventory::iter::<PluginEntry<T>> {
        if entry.name == target {
            selected_plugin = Some((entry.constructor)());
            break;
        }
    }

    let plugin = selected_plugin.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("no matching plugin found for '{target}'"),
        )
    })?;

    let _ = tokio::fs::remove_file(&args.socket).await;

    let listener = tokio::net::UnixListener::bind(&args.socket)?;

    let (stream, _) = listener.accept().await?;
    let transport = std::sync::Arc::new(rapace::StreamTransport::new(stream));
    Ok((plugin, transport))
}

pub async fn run_build_script_mode<T, F, Fut>(force_link: fn(), serve: F)
where
    T: Send + Sync + 'static,
    F: FnOnce(
        T,
        Arc<
            rapace::StreamTransport<
                tokio::io::ReadHalf<tokio::net::UnixStream>,
                tokio::io::WriteHalf<tokio::net::UnixStream>,
            >,
        >,
    ) -> Fut,
    Fut: Future<Output = Result<(), RpcError>>,
{
    let _ = force_link();
    if std::env::args().any(|a| a == "--runner") {
        match run_runner_mode::<T>().await {
            Ok((plugin, transport)) => serve(plugin, transport).await.unwrap(),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
        return;
    }

    prepare_runner_and_emit_env().unwrap();
}
