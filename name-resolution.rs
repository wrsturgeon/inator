fn shit() {
    println!("shit");
}

mod module {
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
