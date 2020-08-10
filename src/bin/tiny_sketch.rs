use std::thread;
use std::time;

fn main() {
    println!("Oy!");
    thread::sleep(time::Duration::as_micros(5));
    println!("Ay!");
}

// fn main() {
//     let mut i: usize = 0;
//     println!("{}", i as isize - 1);
//     println!("{}", ((i as isize - 1) % 3 as isize));
//     // i = ((i as isize - 1) % 3 as isize) as usize;
//     // println!("{}", i);
//     // i = ((i as isize - 1) % 3 as isize) as usize;
//     // println!("{}", i);
//     // i = ((i as isize - 1) % 3 as isize) as usize;
//     // println!("{}", i);
//     // i = ((i as isize - 1) % 3 as isize) as usize;
//     // println!("{}", i);
// }
