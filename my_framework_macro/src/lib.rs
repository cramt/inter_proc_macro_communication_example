use plugin_api::PluginServiceClient;
use proc_macro::TokenStream;

#[proc_macro]
pub fn my_framework(input: TokenStream) -> TokenStream {
    let runner_path = match std::env::var("MY_FRAMEWORK_RUNNER") {
        Ok(p) => p,
        Err(_) => {
            return "compile_error!(\"MY_FRAMEWORK_RUNNER not set; ensure build.rs runs and sets it.\");"
                .parse()
                .unwrap();
        }
    };

    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let socket_path = temp_dir.path().join("rapace.sock");
    let socket_path_str = socket_path.to_str().unwrap().to_string();

    let mut cmd = std::process::Command::new(&runner_path);
    cmd.arg("--runner")
        .arg("hello") // fixed plugin name for this minimal example
        .arg("--socket")
        .arg(&socket_path_str)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("compile_error!(\"failed to spawn runner: {e}\");");
            return msg.parse().unwrap();
        }
    };

    let input_str = input.to_string();
    let socket_path_moved = socket_path_str.clone();

    // Run async code in a separate thread to avoid conflicting with proc-macro runtime if any
    let result = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            let mut attempts = 0;
            loop {
                if attempts > 200 {
                    return Err("timed out waiting for runner socket".to_string());
                }

                // rapace::transport::unix::UnixClient::connect returns a transport
                match tokio::net::UnixStream::connect(&socket_path_moved).await {
                    Ok(stream) => {
                        let transport = std::sync::Arc::new(rapace::StreamTransport::new(stream));
                        let session = std::sync::Arc::new(rapace::RpcSession::new(transport));
                        let s = session.clone();
                        tokio::spawn(async move {
                            // Ensure to handle the result to avoid warning
                            let _ = s.run().await;
                        });
                        let client = PluginServiceClient::new(session);
                        let res = client
                            .transform(input_str)
                            .await
                            .map_err(|e| format!("rpc error: {:?}", e));
                        return res;
                        // Rapace generated client for String return type returns Result<String, RpcError>.
                        // So res is Result<String, String> (where Err is formatted RpcError).
                        // Previously it was Result<Result<String, String>, RpcError>.
                        // Wait, thread returns Result<Result<String, String>, String> previously.
                        // Now thread returns Result<String, String>.
                        // match result handles Ok(Ok(Ok(s))).

                        // Let's adjust thread return type to simply Result<String, String>.
                        // And adjust match block.
                    }
                    Err(_) => {
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                        attempts += 1;
                    }
                }
            }
        })
    })
    .join();

    let _ = child.kill();
    let _ = child.wait();

    match result {
        Ok(Ok(s)) => match s.parse() {
            Ok(ts) => ts,
            Err(e) => {
                let err_msg =
                    format!("compile_error!(\"runner output was not valid Rust tokens: {e:?}\");");
                err_msg.parse().unwrap()
            }
        },
        Ok(Err(e)) => format!("compile_error!(\"plugin/rpc failed: {:?}\");", e)
            .parse()
            .unwrap(),
        Err(e) => format!("compile_error!(\"client thread panicked: {:?}\");", e)
            .parse()
            .unwrap(),
    }
}
