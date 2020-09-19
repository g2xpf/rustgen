use rustgen_macros::gen;

fn main() {
    _main()
}

gen! {
    #if a {
        fn _main() {
            println!("a");
        }
    } #else #if c.e {
        fn _main() {
            println!("b");
        }
    } #else #if c.d {
        fn _main() {
            println!("c");
        }
    }
}
