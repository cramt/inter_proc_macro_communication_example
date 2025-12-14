use plugin_api::{Plugin, PluginEntry};

/// A trivial example plugin that wraps input tokens into a function.
pub struct Hello;

impl Plugin for Hello {
    fn transform(&self, input: &str) -> Result<String, String> {
        Ok(format!("pub fn hello_plugin() {{ {} }}", input))
    }
}

// Constructor function (a plain `fn` is a compile-time constant).
fn make_hello() -> Box<dyn Plugin> {
    Box::new(Hello)
}

plugin_api::inventory::submit!(PluginEntry {
    name: "hello",
    constructor: make_hello,
});

pub fn force_link() {}
