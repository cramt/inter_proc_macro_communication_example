//! Proc-macro facade for the `hello` plugin.
//!
//! This crate provides a small proc-macro that delegates token transformation to
//! the menoetius runner via the library helper in `menoetius_macro`.
//!
//! Usage (in an example consumer crate):
//! ```ignore
//! use my_plugin_hello_macro::hello_plugin;
//!
//! hello_plugin!(println!("Hello from plugin!"););
//! ```
use menoetius_macro::get_plugin;
use my_plugin_hello::MyPluginClient;
use proc_macro::TokenStream;
use tokio::runtime::Builder;

/// Proc-macro that asks the `hello` plugin to transform the input token stream.
///
/// It delegates the heavy lifting to the library helper `menoetius_macro::transform_with_runner`.
/// On success the returned string is parsed as Rust tokens and returned. On any error a
/// `compile_error!` invocation is emitted instead.
///
/// Because proc-macros are synchronous, we create a short-lived Tokio runtime here,
/// run the async helper to completion, then shut the runtime down. This keeps the
/// core helper `transform_with_runner` async and runtime-agnostic.
#[proc_macro]
pub fn hello_plugin(input: TokenStream) -> TokenStream {
    let rt = Builder::new_multi_thread()
        .enable_all()
        .worker_threads(1)
        .build()
        .expect("failed to create short-lived tokio runtime");
    let result = rt.block_on(async move {
        let session = get_plugin("hello").await.expect("tesdting");

        // Drive the session in the background to ensure the RPC subsystem runs.
        let s = session.clone();
        tokio::spawn(async move {
            let _ = s.run().await;
        });

        // Construct the generated client and call the transform RPC.
        // The generated client type is expected to be exported by the framework crate.
        let client = MyPluginClient::new(session);
        let res = client.transform(input.to_string()).await.expect("testing");

        res
    });

    match result.parse::<TokenStream>() {
        Ok(ts) => ts,
        Err(e) => {
            let msg =
                format!("compile_error!(\"runner output was not valid Rust tokens: {e:?}\");");
            msg.parse().unwrap_or_else(|_| {
                // As a last resort, emit a very simple compile_error.
                "compile_error!(\"invalid runner output and failed to form error\");"
                    .parse()
                    .unwrap()
            })
        }
    }
}
