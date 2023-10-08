#[allow(dead_code)]
fn shit() {
    println!("shit");
}

mod module {
    #[allow(dead_code)]
    fn shit() {
        panic!("Wrong function");
    }

    pub fn main() {
        use super::*; // Comment this out and see what happens!
        shit()
    }
}

fn main() {
    module::main()
}
