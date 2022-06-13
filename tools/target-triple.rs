/*
File: target-triple.rs
Author: Chase Ruskin
Abstract: 
    Basic executable to extract the current machine's "target triple".
Usage:   
    `rustc target-triple.rs --out-dir DIR`
*/
use std::env;

fn main() {
    println!("{}-{}", env::consts::ARCH, env::consts::OS);
}