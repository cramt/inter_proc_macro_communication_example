pub use inventory;

pub struct PluginEntry<T: Sync + Send + 'static> {
    pub name: &'static str,
    pub constructor: fn() -> T,
}

impl<T: Sync + Send + 'static> inventory::Collect for PluginEntry<T> {
    fn registry() -> &'static inventory::Registry {
        static REGISTRY: inventory::Registry = inventory::Registry::new();
        &REGISTRY
    }
}
