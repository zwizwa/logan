extern crate la;

fn main() {
    let samplerate = 2000000us;
    let baud = 115200us;
    // let baud = 110000us;
    
    let mut uart = la::uart::init(la::uart::Config {
        period:  samplerate / baud,
        nb_bits: 8,
        channel: 0,
    });
    // uart::test(&mut uart);
    for b in la::decode(&mut uart, la::io::stdin8()) {
        //println!("{}", (b as u8) as char);
        la::io::write_byte(b as u8);
    }
}
