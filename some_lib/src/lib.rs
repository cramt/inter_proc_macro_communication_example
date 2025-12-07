proc_macro_a::proc_macro_a!();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        panic!("{:?}", result())
    }
}
