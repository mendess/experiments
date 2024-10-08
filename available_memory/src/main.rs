use available_memory::*;

fn main() {
    println!("Hello, world!: {:?}", ram_readln::<1024>());
    println!("Hello, world!: {:?}", ram_streaming::<1>());
    println!("Hello, world!: {:?}", ram_streaming::<2>());
    println!("Hello, world!: {:?}", ram_streaming::<3>());
    println!("Hello, world!: {:?}", ram_streaming::<5>());
    println!("Hello, world!: {:?}", ram_streaming::<8>());
    println!("Hello, world!: {:?}", ram_streaming::<13>());
    println!("Hello, world!: {:?}", ram_streaming::<1024>());
    println!("Hello, world!: {:?}", ram_streaming::<4096>());
}
