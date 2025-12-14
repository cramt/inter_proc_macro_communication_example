fn main() {
    // If invoked as runner by the proc macro, run in runner mode.
    if std::env::args().any(|a| a == "--runner") {
        // Force link plugin crates so their inventory submissions are present.
        let _ = &my_plugin_hello::force_link();

        if let Err(e) = my_framework_builder::run_as_runner() {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return;
    }

    // Normal build script mode: prepare runner and expose env var
    let _ = &my_plugin_hello::force_link(); // also force-link in normal mode
    my_framework_builder::prepare_runner_and_emit_env().unwrap();
}
