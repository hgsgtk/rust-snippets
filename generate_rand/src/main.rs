extern crate rand;

use rand::Rng;

fn main() {
    let mut rng = rand::thread_rng();
    println!("i32: {}, u32: {}", rng.gen::<i32>(), rng.gen::<u32>());
}
