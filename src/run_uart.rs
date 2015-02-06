#![feature(core)]
extern crate la;
fn main() {
    let samplerate = 8000000us;
    let baud = 9600us;
    
    let mut uart = la::uart::init(la::uart::Config {
        period:  samplerate / baud,
        nb_bits: 8,
        channel: 3,
    });
    // uart::test(&mut uart);
    for b in la::apply(&mut uart, la::io::stdin8()) {
        print!("{}", (b as u8) as char);
    }
}
