#![feature(core)]
#![feature(box_syntax)]
extern crate la;
use la::tick::apply;
use la::tick::syncser::{SyncSer,Config,init};

/* Currently returning a sequence with closures is not possible
without workarounds, so use a macro.  What would help is Box<Fn>
implementing <Fn> to use boxed closures, or abstract return types to
allow unboxed closures. */

macro_rules! test_seq {
    ($c: expr, $data_in: expr, $period: expr) => (
        $data_in.iter()
        .flat_map(|&data|
                  // expand data word into bits
                  (0..$c.nb_bits).flat_map(move |shift| {
                      let bit = (data >> ($c.nb_bits - 1 - shift)) & 1;
                      // expand bit into clocked bit sequence
                      (0..2).map(move |clock|
                                 ($c.frame_active                 << $c.frame_channel) |
                                 (($c.clock_polarity ^ clock ^ 1) << $c.clock_channel) |
                                 (bit                             << $c.data_channel))
                  })
                  // follow with 1 bit frame release
                  .chain((0..1).map(|_|
                                    (($c.frame_active ^ 1) << $c.frame_channel) |
                                    ($c.clock_polarity     << $c.clock_channel)
                                    ))
                  )
            
        // oversample
        .flat_map(|bus|
                  (0..$period).map(move |_| bus))
        )
}

fn test_test_seq(c: &Config) {
    for bus in test_seq!(c, [0x55], 1) {
        println!("{:01$b}", bus, 3);
    }
}

fn test_vec(syncser: &mut SyncSer, data_in: Vec<usize>, period: usize) {
    let c = syncser.config;
    let data_out: Vec<_> =
        decode(syncser,
               test_seq!(c, data_in, period)
               ).collect();
    assert_eq!(data_out, data_in);
}
use std::iter::{count};

fn test_configs() {
    let nb_bits = 8;
    for edge in (0..2) {
        for period in (1..10) {
            let mut syncser = init(Config {
                data_channel: 0,
                clock_channel: 1,
                frame_channel: 2,
                frame_enable: true,
                clock_edge: edge,
                clock_polarity: edge ^ 1,
                frame_active: 0,
                frame_timeout: 0,
                timeout_enable: false,
                nb_bits: nb_bits,
            });
            let n = 1 << nb_bits;
            let s = count(n-1,-1).take(n);
            test_test_seq(&syncser.config);
            test_vec(&mut syncser, s.collect(), period);
        }
    }
    println!("syncser OK");
}

fn main() {
    test_configs();
}

#[test]
fn run_tests() {
    main()
}
