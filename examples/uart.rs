extern crate la;

use la::sm::{uart,apply};
use la::io::{stdin8,write_byte};

fn main() {
    let samplerate = 2000000usize;
    let baud = 115200usize;
    // let baud = 110000u32;
    
    let mut uart = uart::init(uart::Config {
        period:  samplerate / baud,
        nb_bits: 8,
        channel: 0,
    });
    // uart::test(&mut uart);
    for b in apply(&mut uart, stdin8()) {
        //println!("{}", (b as u8) as char);
        write_byte(b as u8);
    }
}
