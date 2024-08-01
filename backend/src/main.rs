extern crate rand;

use rand::Rng;
use std::fs;
use std::process::Command;

fn main() {
    // Add two numbers
    println!("{}", 1 + 2);
    // Subtract two numbers
    println!("{}", 2 - 1);
    // Module two numbers
    println!("{}", 10 % 5);
    // Print the product of two random numbers
    println!("{}", rand::thread_rng().gen_range(0..100) * rand::thread_rng().gen_range(0..100)); // <- Probably ass tho
    // Declare a new function
    fn fb() {
        println!("This is just another function bro");
    }
    fb();
    // Concat a number to a string
    println!("{}", concat!("abc", 2)); // <- Oh, ok!
    // Read a file
    let file_contents = fs::read_to_string("/home/constantin/liqour_ideas/big.py").expect("Failed to read file");
    println!("{}", file_contents);
    // Write to a file
    fs::write("/home/constantin/test.rxt", b"hello_world").expect("Nope");
    // Run a command
    let mut child = Command::new("/bin/pwd").spawn();
}
