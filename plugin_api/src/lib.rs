pub trait Plugin: Sync + Send + 'static {
    fn transform(&self, input: &str) -> Result<String, String>;
}

pub use inventory;

pub struct PluginEntry {
    pub name: &'static str,
    pub constructor: fn() -> Box<dyn Plugin>,
}

inventory::collect!(PluginEntry);
