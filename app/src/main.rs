use my_plugin_hello_macro::hello_plugin;

hello_plugin!(println!("Hello from plugin!"););

fn main() {
    hello_plugin();
}
