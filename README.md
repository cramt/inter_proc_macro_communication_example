# Inter-Proc-Macro Plugin Runner (TL;DR)

**Goal:** Run user-written Rust “plugins” during **proc-macro expansion** without requiring them to precompile to WASM or ship dynamic libs.

## How it works (short)

1.  **Plugins** define their own trait/shape and register a const-friendly entry using the framework's generic `PluginEntry<T>` type. The framework only provides the inventory collection and process/IPC helpers so the concrete plugin surface area can be chosen by each plugin crate.

    ```rust
    // In the plugin crate: define the plugin trait the plugin implements.
    pub trait MyPlugin: Sync + Send + 'static {
        fn transform(&self, input: &str) -> Result<String, String>;
    }

    // Constructor must be a plain `fn` so it is a compile-time constant.
    fn make_hello() -> Box<dyn MyPlugin> { Box::new(Hello) }

    // Submit a PluginEntry where the concrete stored type is `Box<dyn MyPlugin>`.
    // The framework's inventory is generic, so plugin crates submit the appropriate
    // `PluginEntry::<Box<dyn MyPlugin>>`.
    menoetius::inventory::submit!(menoetius::PluginEntry::<Box<dyn MyPlugin>> {
        name: "hello",
        constructor: make_hello,
    });
    ```

    The function pointer is still a compile-time constant; at runtime the framework iterates the collected `PluginEntry<T>` entries for the expected `T` (for example `Box<dyn MyPlugin>`) and calls the constructors.

2.  **Build script (`app/build.rs`)** links plugin crates as **build-dependencies**, then:
    *   **Normal mode:** copies its own executable to `OUT_DIR/menoetius_runner` and sets `MENOETIUS_RUNNER` env var.
    *   **Runner mode (`--runner`)**: reads tokens from `stdin`, picks a plugin by `name`, runs `transform()`, writes tokens to `stdout`.

3.  **Proc macro** spawns the runner, streams input tokens, receives output tokens, and returns them.

## Quick start

```bash
cargo run -p app
```

**Expected output:**

    Hello from plugin!

## Project layout

    menoetius/             # Framework: generic `PluginEntry<T>` + inventory collection and runner helpers (no fixed `Plugin` trait)
    my_plugin_hello/       # Example plugin: defines its own trait `MyPlugin`, implements it and `submit!(PluginEntry::<Box<dyn MyPlugin>>{...})`
    menoetius_build/       # prepare_runner_and_emit_env() + async `run_build_script_mode::<T>()` / runner helpers
    menoetius_macro/       # helper library: async `transform_with_runner()` that performs process setup and RPC
    my_plugin_hello_macro/ # proc-macro crate that invokes the helper with a short-lived Tokio runtime
    app/                   # consumer crate: build.rs (delegates to menoetius_build) + main.rs

## Key snippets

**Register a plugin:**

```rust
fn make_hello() -> Box<dyn MyPlugin> { Box::new(Hello) }

menoetius::inventory::submit!(PluginEntry::<Box<dyn MyPlugin>> {
    name: "hello",
    constructor: make_hello,
});

pub fn force_link() {} // called from build.rs so the crate is kept by the linker
```

**Build script (dual-mode):**

```rust
fn main() {
    // Build scripts are synchronous; run the async build-script helper via a short-lived Tokio runtime.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime for build script");

    rt.block_on(async {
        // Call the generic async helper for the plugin trait implemented by the plugin crate.
        // The plugin crate must export `force_link()` and its trait (here `MyPlugin`).
        menoetius_build::run_build_script_mode::<Box<dyn my_plugin_hello::MyPlugin>>(my_plugin_hello::force_link).await;
    });
}
```

**Proc macro (call runner):**

```rust
// Most users will invoke a plugin-specific proc-macro rather than calling the runner directly.
// Example (in the consumer crate source):
hello_plugin!(println!("Hello from plugin!"););

// If you need to call the async helper directly from an async context, use the helper:
// let result = menoetius_macro::transform_with_runner("hello", input_string).await?;
```

## Add more plugins

*   Create another crate, define the plugin trait you want (for example `MyPlugin`), implement it and provide a `constructor: fn() -> Box<dyn MyPlugin>`.
*   Submit the entry as `menoetius::inventory::submit!(menoetius::PluginEntry::<Box<dyn MyPlugin>> { name: "world", constructor });`.
*   Export `pub fn force_link()` in that crate and add the crate to `app`’s **`[build-dependencies]`**. In `build.rs` pass its `force_link` function to `menoetius_build::run_build_script_mode::<Box<dyn your_crate::YourTrait>>(your_crate::force_link).await` (or use a short-lived runtime wrapper if `build.rs` is synchronous).
*   From proc-macros, call the helper (or a plugin-specific proc-macro) and select the plugin by name (e.g. `hello_plugin!(...)`).

## Caveats

*   **Build-time code execution:** plugins run on the host during build—treat them as trusted code.
*   **Performance:** runner is a process per macro call; consider caching or a long-lived runner if needed.
*   **Environments like docs.rs:** may restrict spawning—gate with features or provide a no-op fallback.
