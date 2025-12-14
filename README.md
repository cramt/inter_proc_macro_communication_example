# Inter-Proc-Macro Plugin Runner (TL;DR)

**Goal:** Run user-written Rust “plugins” during **proc-macro expansion** without requiring them to precompile to WASM or ship dynamic libs.

## How it works (short)

1.  **Plugins** implement a `Plugin` trait and register via `inventory::submit!` a **const-friendly entry**:
    ```rust
    pub struct PluginEntry {
      pub name: &'static str,
      pub constructor: fn() -> Box<dyn Plugin>,
    }
    ```
    The function pointer is a compile-time constant; at runtime we call it to construct the plugin.

2.  **Build script (`app/build.rs`)** links plugin crates as **build-dependencies**, then:
    *   **Normal mode:** copies its own executable to `OUT_DIR/my_framework_runner` and sets `MY_FRAMEWORK_RUNNER` env var.
    *   **Runner mode (`--runner`)**: reads tokens from `stdin`, picks a plugin by `name`, runs `transform()`, writes tokens to `stdout`.

3.  **Proc macro** spawns the runner, streams input tokens, receives output tokens, and returns them.

## Quick start

```bash
cargo run -p app
```

**Expected output:**

    Hello from plugin!

## Project layout

    plugin_api/            # Plugin trait + PluginEntry + inventory::collect!(PluginEntry)
    my_plugin_hello/       # Example plugin: submit!(PluginEntry { name: "hello", constructor: make_hello })
    my_framework_builder/  # prepare_runner_and_emit_env() + run_as_runner()
    my_framework_macro/    # proc_macro that spawns the runner and returns transformed tokens
    app/                   # consumer crate: build.rs (dual-mode) + main.rs

## Key snippets

**Register a plugin:**

```rust
fn make_hello() -> Box<dyn Plugin> { Box::new(Hello) }

plugin_api::inventory::submit!(PluginEntry {
    name: "hello",
    constructor: make_hello,
});

pub fn force_link() {} // called from build.rs so the crate is kept by the linker
```

**Build script (dual-mode):**

```rust
fn main() {
    if std::env::args().any(|a| a == "--runner") {
        my_plugin_hello::force_link();
        my_framework_builder::run_as_runner().unwrap();
        return;
    }
    my_plugin_hello::force_link();
    my_framework_builder::prepare_runner_and_emit_env().unwrap();
}
```

**Proc macro (call runner):**

```rust
let runner = std::env::var("MY_FRAMEWORK_RUNNER").expect("runner not set");
let out = std::process::Command::new(&runner)
    .arg("--runner").arg("hello")
    .stdin(Stdio::piped()).stdout(Stdio::piped())
    .spawn()?.wait_with_output()?;
let tokens = String::from_utf8(out.stdout)?.parse()?;
```

## Add more plugins

*   Create another crate, implement `Plugin`, add a `constructor`, `submit!` with `name: "world"`, and export `pub fn force_link() {}`.
*   Add it to `app`’s **`[build-dependencies]`** and call `my_plugin_world::force_link()` in `build.rs`.
*   Pass `"world"` to the runner from the macro to select it.

## Caveats

*   **Build-time code execution:** plugins run on the host during build—treat them as trusted code.
*   **Performance:** runner is a process per macro call; consider caching or a long-lived runner if needed.
*   **Environments like docs.rs:** may restrict spawning—gate with features or provide a no-op fallback.
