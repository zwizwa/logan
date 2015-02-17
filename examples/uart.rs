#![feature(core)]
#![feature(io)]
extern crate la;

fn main() {
    use std::old_io;
    let mut out = old_io::stdout();

    let samplerate = 2000000us;
    let baud = 115200us;
    
    let mut uart = la::uart::init(la::uart::Config {
        period:  samplerate / baud,
        nb_bits: 8,
        channel: 0,
    });
    // uart::test(&mut uart);
    for b in la::decode(&mut uart, la::io::stdin8()) {
        if true {
            let bs = [b as u8];
            // match out.write_all(format!(" {0:x}", b).as_bytes())
            match out.write_all(&bs) {
                Err(err) => panic!("{}",err),
                Ok(_) => (),
            }
            match out.flush() {
                Err(err) => panic!("{}",err),
                Ok(_) => (),
            }
        }
        else {
            // Print doesn't work.  I'm probably doing something wrong
            // but it seems that stdin isn't getting any bytes which
            // is weird.  Internal bug?
            print!("{}", (b as u8) as char);
        }
    }
}
