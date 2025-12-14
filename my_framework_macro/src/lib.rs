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

    let input_str = input.to_string();
    let mut cmd = std::process::Command::new(&runner_path);
    cmd.arg("--runner")
        .arg("hello") // fixed plugin name for this minimal example
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("compile_error!(\"failed to spawn runner: {e}\");");
            return msg.parse().unwrap();
        }
    };

    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        if let Err(e) = stdin.write_all(input_str.as_bytes()) {
            let msg = format!("compile_error!(\"failed to write to runner stdin: {e}\");");
            return msg.parse().unwrap();
        }
    }

    let out = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => {
            let msg = format!("compile_error!(\"runner failed to run: {e}\");");
            return msg.parse().unwrap();
        }
    };

    if !out.status.success() {
        let msg = format!(
            "compile_error!(\"runner error: status {status}\");",
            status = out.status
        );
        return msg.parse().unwrap();
    }

    let out_str = match String::from_utf8(out.stdout) {
        Ok(s) => s,
        Err(_) => {
            return "compile_error!(\"runner produced non-UTF8 output\");"
                .parse()
                .unwrap();
        }
    };

    match out_str.parse() {
        Ok(ts) => ts,
        Err(_) => {
            let sanitized = out_str.replace('\\', "\\\\").replace('\"', "\\\"");
            let msg = format!(
                "compile_error!(\"runner output was not valid Rust tokens: {sanitized}\");"
            );
            msg.parse().unwrap()
        }
    }
}
