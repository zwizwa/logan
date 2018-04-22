#![feature(core)]
use la::tick::{diff,apply};
use la::io::stdin8;
extern crate la;
fn main() {
    let mut diff = diff::init();
    for b in apply(&mut diff, stdin8()) {
        println!("{:01$x}",b,2);
        // la::io::write_byte(b);
    }
}
