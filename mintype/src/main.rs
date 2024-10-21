mod decl {
    use declarative::min_type;

    type N = min_type!(300, u16, u8, i64, u32);
    type N10 = min_type!(10, u16, u8);
    // type NError = min_type!(1_000_000, u16, u8, u8);

    pub fn main() {
        let a: N = 300;
        println!("{a} :: {}", std::any::type_name::<N>());
        let a: N10 = 10;
        println!("{a} :: {}", std::any::type_name::<N10>());
        // let c: NError = 10;
        // println!("() :: {}", std::any::type_name::<NError>());
    }
}

mod proc {
    use procedural::min_type;

    type N = min_type!(300, u16, u8, i64, u32);
    type N10 = min_type!(10, u16, u8);
    // type NError = min_type!(1_000_000, u16, u8, u8);

    pub fn main() {
        let a: N = 300;
        println!("{a} :: {}", std::any::type_name::<N>());
        let a: N10 = 10;
        println!("{a} :: {}", std::any::type_name::<N10>());
        // let c: NError = 10;
        // println!("() :: {}", std::any::type_name::<NError>());
    }
}

fn main() {
    decl::main();
    proc::main();
}
