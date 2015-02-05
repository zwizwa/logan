#![feature(core)]
#![feature(io)]
extern crate la;
fn main() {
    use std::old_io;
    let mut out = old_io::stdout();
    
    let diff = la::diff::init();
    for b in la::la::proc_map(diff, la::io::stdin8()) {
        match out.write_all(format!(" {0:x}", b).as_bytes()) {
            Err(err) => panic!("{}",err),
            Ok(_) => (),
        }
        match out.flush() {
            Err(err) => panic!("{}",err),
            Ok(_) => (),
        }
    }
}
