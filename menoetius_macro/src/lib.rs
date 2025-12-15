use miette::Diagnostic;
use std::process::Stdio;
use thiserror::Error;
use tokio::process::Command;

#[derive(Error, Diagnostic, Debug)]
pub enum GetPluginError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("getting socked timed oout")]
    Timeout,
    #[error(transparent)]
    VarError(#[from] std::env::VarError),
}

pub async fn get_plugin(
    plugin: &str,
) -> Result<
    std::sync::Arc<
        rapace::RpcSession<
            rapace::StreamTransport<
                tokio::io::ReadHalf<tokio::net::UnixStream>,
                tokio::io::WriteHalf<tokio::net::UnixStream>,
            >,
        >,
    >,
    GetPluginError,
> {
    let runner_path = std::env::var("MENOETIUS_RUNNER")?;

    let temp_dir = tempfile::tempdir()?.keep();
    let socket_path = temp_dir.join("rapace.sock");
    let socket_path_str = socket_path.to_str().unwrap().to_string();

    // Spawn the runner using Tokio's async process API.
    let mut cmd = Command::new(&runner_path);
    cmd.arg("--runner")
        .arg(plugin)
        .arg("--socket")
        .arg(&socket_path_str)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    println!("{runner_path} --runner {plugin} --socket {socket_path_str}");
    let mut child = cmd.spawn()?;

    // Connect to the runner socket, retrying until it appears or we time out.
    let mut attempts: u32 = 0;
    let session = loop {
        if attempts > 200 {
            // Give up and clean up child
            let _ = child.kill().await.ok();
            let _ = child.wait().await.ok();
            return Err(GetPluginError::Timeout);
        }

        match tokio::net::UnixStream::connect(&socket_path_str).await {
            Ok(stream) => {
                let transport = std::sync::Arc::new(rapace::StreamTransport::new(stream));
                let session = std::sync::Arc::new(rapace::RpcSession::new(transport));
                break session;
            }
            Err(_) => {
                attempts += 1;
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                continue;
            }
        }
    };
    Ok(session)
}
