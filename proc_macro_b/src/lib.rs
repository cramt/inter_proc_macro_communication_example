extern crate proc_macro;
use std::process::Command;

use proc_macro::TokenStream;

#[proc_macro]
pub fn proc_macro_b(item: TokenStream) -> TokenStream {
    let stdout = Command::new(item.to_string().replace('"', ""))
        .output()
        .unwrap()
        .stdout;

    let stdout = String::from_utf8(stdout).unwrap();

    format!("fn result() -> &'static str {{\"{}\"}}", stdout)
        .parse()
        .unwrap()
}
