#![feature(core)]
extern crate la;
use la::decode;
use la::uart::{Uart,Config,init};

fn frame(nb_bits: usize, value: usize) -> usize {
    (value | (1 << nb_bits)) << 1
}
fn test_vec(uart: &mut Uart, data_in: Vec<usize>) {
    let c = uart.config;
    let data_out: Vec<_> =
        decode(uart,
               data_in.iter()
               // expand data word to UART frame bit sequence
               .flat_map(|&data| (0..c.nb_bits+2).map(move |shift| (frame(c.nb_bits, data) >> shift) & 1))
               // shift it to the correct channel on the bus
               .map(|bit| bit << c.channel)
               // oversample bus sequence
               .flat_map(|bus|   (0..c.period).map(move |_| bus))
               ).collect();
    assert_eq!(data_out, data_in);
}
use std::iter::{count};

fn test_configs() {
    for nb_bits in (7..10) {
        for channel in (0..3) {
            for period in (1..10) {
                let mut uart = init(Config {
                    period:  period,
                    nb_bits: nb_bits,
                    channel: channel,
                });
                let n = 1 << nb_bits;
                let s = count(n-1,-1).take(n);
                test_vec(&mut uart, s.collect());
            }
        }
    }
    println!("test1 OK");
}

fn main() {
    test_configs();
}
