use menoetius::PluginEntry;

#[allow(async_fn_in_trait)]
#[rapace::service]
pub trait MyPlugin {
    async fn transform(&self, input: String) -> String
    where
        Self: Sized;
}

/// A trivial example plugin that wraps input tokens into a function.
pub struct Hello;

impl MyPlugin for Hello {
    async fn transform(&self, input: String) -> String {
        format!("pub fn hello_plugin() {{ {} }}", input)
    }
}

// Constructor function (a plain `fn` is a compile-time constant).
fn make_hello() -> Hello {
    Hello
}

type MyPluginEntry = PluginEntry<Hello>;

menoetius::inventory::submit!(MyPluginEntry {
    name: "hello",
    constructor: make_hello,
});

/// Called from build.rs to force linkage of this crate into the build script
/// executable so the inventory submit! actually runs.
pub fn force_link() {}
