#![feature(core)]
extern crate la;
fn main() {
    let mut diff = la::diff::init();
    for b in la::decode(&mut diff, la::io::stdin8()) {
        println!("{:01$x}",b,2);
        // la::io::write_byte(b);
    }
}
