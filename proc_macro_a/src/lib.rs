use std::path::PathBuf;
extern crate proc_macro;
use proc_macro::TokenStream;

fn plugin_path() -> PathBuf {
    PathBuf::from(env!("OUT_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("plugin")
        .to_path_buf()
}

#[proc_macro]
pub fn proc_macro_a(_item: TokenStream) -> TokenStream {
    format!(
        "proc_macro_b::proc_macro_b!(\"{}\");",
        plugin_path().to_str().unwrap()
    )
    .parse()
    .unwrap()
}
