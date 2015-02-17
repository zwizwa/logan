extern crate la;

fn main() {
    use la::{slip,uart,decode};
    let samplerate = 2000000us;
    let baud = 115200us;
    

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
     
    for packet in decode(&mut slip, decode(&mut uart, la::io::stdin8())) {
        slip::print(packet);
    }
}
