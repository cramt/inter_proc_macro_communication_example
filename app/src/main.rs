use my_framework_macro::my_framework;

my_framework!(println!("Hello from plugin!"););

fn main() {
    hello_plugin();
}
