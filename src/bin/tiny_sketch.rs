use std::env;
use std::path::Path;
use path_absolutize::*;

fn main() {
    let cur_dir = env::current_dir().unwrap();
    println!("{}", cur_dir.to_str().unwrap());

    let path = Path::new(r"./dkljflkd\lkdjdkl.jpg");
    println!("Is relative: {:?}", path.is_relative());
    println!("Is dir: {:?}", path.is_dir());



    // let path = Path::new(r".\src\");
    println!("{}", path.to_str().unwrap());
    println!("{}\n", path.absolutize().unwrap().to_str().unwrap());

    // let path = Path::new(r"\src\");
    // println!("{}", path.to_str().unwrap());
    // println!("{}\n", path.absolutize().unwrap().to_str().unwrap());

    // let path = Path::new(r"src\");
    // println!("{}", path.to_str().unwrap());
    // println!("{}\n", path.absolutize().unwrap().to_str().unwrap());

    // let path = Path::new(r"C:\Users\Andy\OneDrive\Dokumenter\Andy hjemme\Programmering\Projekte\bitgeon\src");
    // println!("{}", path.to_str().unwrap());
    // println!("{}\n", path.absolutize().unwrap().to_str().unwrap());
}

