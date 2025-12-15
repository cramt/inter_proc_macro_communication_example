pub trait Plugin: Sync + Send + 'static {
    // keeping the original trait for now so existing plugins compile,
    // but the IPC will use the service trait which wraps this.
    fn transform(&self, input: &str) -> Result<String, String>;
}

#[rapace::service]
#[allow(async_fn_in_trait)]
pub trait PluginService {
    async fn transform(&self, input: String) -> String;
}

pub use inventory;

pub struct PluginEntry {
    pub name: &'static str,
    pub constructor: fn() -> Box<dyn Plugin>,
}

inventory::collect!(PluginEntry);
