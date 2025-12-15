use my_plugin_hello::{Hello, MyPluginServer};

fn main() {
    // Build scripts are synchronous; run the async build-script helper via a short-lived Tokio runtime.
    // We call the generic helper with the plugin trait object type implemented by the plugin crate.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime for build script");

    rt.block_on(async {
        // Note: the generic type here is the plugin trait implemented by the plugin crate.
        // This matches how the plugin submitted its inventory entry as PluginEntry::<Box<dyn MyPlugin>>.
        menoetius_build::run_build_script_mode::<Hello, _, _>(
            my_plugin_hello::force_link,
            |plugin, transport| MyPluginServer::new(plugin).serve(transport),
        )
        .await;
    });
}
