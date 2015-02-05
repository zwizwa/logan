#![feature(core)]
extern crate la;
fn main() {
    let samplerate = 4000000us;
    let baud = 9600us;
    
    let uart = la::uart::init(la::uart::Config {
        period:  samplerate / baud,
        nb_bits: 8,
        channel: 3,
    });
    // uart::test(&mut uart);
    for b in la::la::proc_map(uart, la::io::stdin8()) {
        print!("{}", (b as u8) as char);
    }
}
