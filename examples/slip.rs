extern crate la;

use la::sm::{slip,uart,apply};
use la::io::{stdin8};

fn main() {

    let samplerate = 1000000usize;
    let baud = 115200usize;

    let mut slip = slip::init(slip::Config {
        end: 0x0D,
        esc: 0x0C,
        esc_end: 0x0B,
        esc_esc: 0x0A,
    });
    
    let mut uart = uart::init(uart::Config {
        period:  samplerate / baud,
        nb_bits: 8,
        channel: 0,
    });
     
    for packet in apply(&mut slip,
                  apply(&mut uart,
                        stdin8())) {
        slip::print(packet);
    }
}
