#![feature(core)]
extern crate la;
use la::decode;
use la::syncser::{SyncSer,Config,init};

fn test_vec(syncser: &mut SyncSer, data_in: Vec<usize>, nb_bits: usize, period: usize) {
    let c = syncser.config;
    let data_out: Vec<_> =
        decode(syncser,
               data_in.iter()
               // expand data word into bits
               // FIXME: add frame strobe, polarities
               .flat_map(|&data|
                         (0..nb_bits).flat_map(move |shift| {
                             let bit = (data >> (nb_bits - 1 - shift)) & 1;
                             // expand bit into clocked bit sequence
                             (0..2).map(move |clock|
                                        (clock << c.clock_channel) |
                                        (bit   << c.data_channel))
                         }))
               // oversample
               .flat_map(|bus|
                         (0..period).map(move |_| bus))
               ).collect();
    assert_eq!(data_out, data_in);
}
use std::iter::{count};

fn test_configs() {
    let nb_bits = 8;
    for period in (1..10) {
        let mut syncser = init(Config {
            data_channel: 0,
            clock_channel: 1,
            frame_channel: 2,
            frame_enable: false,
            clock_edge: 0,
            clock_polarity: 0,
            frame_active: 0,
            frame_timeout: 0,
            timeout_enable: false,
            nb_bits: nb_bits,
        });
        let n = 1 << nb_bits;
        let s = count(n-1,-1).take(n);
        test_vec(&mut syncser, s.collect(), nb_bits, period);
    }
    println!("test1 OK");
}

fn main() {
    test_configs();
}

#[test]
fn run_tests() {
    main()
}
