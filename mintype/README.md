# Calculate the minimum type that can hold a number

See example in [./src/main.rs](./src/main.rs).

```rust
mod decl {
    use declarative::min_type;

    type N = min_type!(300, u16, u8, i64, u32);
    type N10 = min_type!(10, u16, u8);

    pub fn main() {
        let a: N = 300;
        println!("{a} :: {}", std::any::type_name::<N>());   // 300 :: u16
        let a: N10 = 10;
        println!("{a} :: {}", std::any::type_name::<N10>()); // 10 :: u8
    }
}

mod proc {
    use procedural::min_type;

    type N = min_type!(300, u16, u8, i64, u32);
    type N10 = min_type!(10, u16, u8);

    pub fn main() {
        let a: N = 300;
        println!("{a} :: {}", std::any::type_name::<N>());   // 300 :: u16
        let a: N10 = 10;
        println!("{a} :: {}", std::any::type_name::<N10>()); // 10 :: u8
    }
}

fn main() {
    decl::main();
    proc::main();
}
```

When that example is expanded you get this:
```rust
mod decl {
    use declarative::min_type;
    type N = <::declarative::CondType<
        {
            (<u16>::MIN as i128) <= (300 as i128)
                && (300 as i128) <= (<u16>::MAX as i128)
        },
        (
            u16,
            ::declarative::CondType<
                {
                    (<u8>::MIN as i128) <= (300 as i128)
                        && (300 as i128) <= (<u8>::MAX as i128)
                },
                (
                    u8,
                    ::declarative::CondType<
                        {
                            (<i64>::MIN as i128) <= (300 as i128)
                                && (300 as i128) <= (<i64>::MAX as i128)
                        },
                        (
                            i64,
                            ::declarative::CondType<
                                {
                                    (<u32>::MIN as i128) <= (300 as i128)
                                        && (300 as i128) <= (<u32>::MAX as i128)
                                },
                                (u32, ::declarative::NoTypeCanHoldValue),
                                ::declarative::NoTypeCanHoldValue,
                            >,
                        ),
                        ::declarative::CondType<
                            {
                                (<u32>::MIN as i128) <= (300 as i128)
                                    && (300 as i128) <= (<u32>::MAX as i128)
                            },
                            (u32, ::declarative::NoTypeCanHoldValue),
                            ::declarative::NoTypeCanHoldValue,
                        >,
                    >,
                ),
                ::declarative::CondType<
                    {
                        (<i64>::MIN as i128) <= (300 as i128)
                            && (300 as i128) <= (<i64>::MAX as i128)
                    },
                    (
                        i64,
                        ::declarative::CondType<
                            {
                                (<u32>::MIN as i128) <= (300 as i128)
                                    && (300 as i128) <= (<u32>::MAX as i128)
                            },
                            (u32, ::declarative::NoTypeCanHoldValue),
                            ::declarative::NoTypeCanHoldValue,
                        >,
                    ),
                    ::declarative::CondType<
                        {
                            (<u32>::MIN as i128) <= (300 as i128)
                                && (300 as i128) <= (<u32>::MAX as i128)
                        },
                        (u32, ::declarative::NoTypeCanHoldValue),
                        ::declarative::NoTypeCanHoldValue,
                    >,
                >,
            >,
        ),
        ::declarative::CondType<
            {
                (<u8>::MIN as i128) <= (300 as i128)
                    && (300 as i128) <= (<u8>::MAX as i128)
            },
            (
                u8,
                ::declarative::CondType<
                    {
                        (<i64>::MIN as i128) <= (300 as i128)
                            && (300 as i128) <= (<i64>::MAX as i128)
                    },
                    (
                        i64,
                        ::declarative::CondType<
                            {
                                (<u32>::MIN as i128) <= (300 as i128)
                                    && (300 as i128) <= (<u32>::MAX as i128)
                            },
                            (u32, ::declarative::NoTypeCanHoldValue),
                            ::declarative::NoTypeCanHoldValue,
                        >,
                    ),
                    ::declarative::CondType<
                        {
                            (<u32>::MIN as i128) <= (300 as i128)
                                && (300 as i128) <= (<u32>::MAX as i128)
                        },
                        (u32, ::declarative::NoTypeCanHoldValue),
                        ::declarative::NoTypeCanHoldValue,
                    >,
                >,
            ),
            ::declarative::CondType<
                {
                    (<i64>::MIN as i128) <= (300 as i128)
                        && (300 as i128) <= (<i64>::MAX as i128)
                },
                (
                    i64,
                    ::declarative::CondType<
                        {
                            (<u32>::MIN as i128) <= (300 as i128)
                                && (300 as i128) <= (<u32>::MAX as i128)
                        },
                        (u32, ::declarative::NoTypeCanHoldValue),
                        ::declarative::NoTypeCanHoldValue,
                    >,
                ),
                ::declarative::CondType<
                    {
                        (<u32>::MIN as i128) <= (300 as i128)
                            && (300 as i128) <= (<u32>::MAX as i128)
                    },
                    (u32, ::declarative::NoTypeCanHoldValue),
                    ::declarative::NoTypeCanHoldValue,
                >,
            >,
        >,
    > as ::declarative::Min>::Output;
    type N10 = <::declarative::CondType<
        { (<u16>::MIN as i128) <= (10 as i128) && (10 as i128) <= (<u16>::MAX as i128) },
        (
            u16,
            ::declarative::CondType<
                {
                    (<u8>::MIN as i128) <= (10 as i128)
                        && (10 as i128) <= (<u8>::MAX as i128)
                },
                (u8, ::declarative::NoTypeCanHoldValue),
                ::declarative::NoTypeCanHoldValue,
            >,
        ),
        ::declarative::CondType<
            {
                (<u8>::MIN as i128) <= (10 as i128)
                    && (10 as i128) <= (<u8>::MAX as i128)
            },
            (u8, ::declarative::NoTypeCanHoldValue),
            ::declarative::NoTypeCanHoldValue,
        >,
    > as ::declarative::Min>::Output;
    pub fn main() {
        let a: N = 300;
        {
            ::std::io::_print(
                format_args!("{1} :: {0}\n", std::any::type_name::< N > (), a),
            );
        };
        let a: N10 = 10;
        {
            ::std::io::_print(
                format_args!("{1} :: {0}\n", std::any::type_name::< N10 > (), a),
            );
        };
    }
}
mod proc {
    use procedural::min_type;
    type N = u16;
    type N10 = u8;
    pub fn main() {
        let a: N = 300;
        {
            ::std::io::_print(
                format_args!("{1} :: {0}\n", std::any::type_name::< N > (), a),
            );
        };
        let a: N10 = 10;
        {
            ::std::io::_print(
                format_args!("{1} :: {0}\n", std::any::type_name::< N10 > (), a),
            );
        };
    }
}
fn main() {
    decl::main();
    proc::main();
}
```
