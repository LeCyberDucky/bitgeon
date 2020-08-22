use std::env;
use std::path::Path;
use path_absolutize::*;

fn main() {
    let pattern = ['a', 'b', 'c'];

    let character = 'a';

    match character {
        pattern.iter().any() => println!("Oy!"),
        'd' => println!("Ay!"),
        _ => println!("Ey!"),
    }
}

