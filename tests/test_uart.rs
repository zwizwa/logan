#![feature(core)]
extern crate la;
use la::tick::{apply,uart};

fn frame(nb_bits: usize, value: usize) -> usize {
    (value | (1 << nb_bits)) << 1
}
fn test_vec(uart: &mut uart::Uart, data_in: Vec<usize>) {
    let c = uart.config;

    // expand data word to UART frame bit sequence
    let data_bits = 
        data_in.iter()
        .flat_map(
            |&data|
            (0..c.nb_bits+2).map(
                move |shift|
                (frame(c.nb_bits, data) >> shift) & 1));

    // shift it to the correct channel on the bus
    let data_channel =
        data_bits.map(|bit| bit << c.channel);
        
    // oversample bus sequence
    let mut data_oversample =
        data_channel
        .flat_map(
            |bus|
            (0..c.period).map(
                move |_|
                bus));

    // decode it
    let data_out: Vec<_> =
        apply(uart, &mut data_oversample).collect();

    assert_eq!(data_out, data_in);
}



fn test_configs() {
    for period in 1..20 {
        println!("{} {}", period, uart::start_delay(period));
    }

    for nb_bits in 7..10 {
        for channel in 0..3 {
            for period in 1..10 {
                let mut uart = uart::init(
                    uart::Config {
                        period:  period,
                        nb_bits: nb_bits,
                        channel: channel,
                    }
                );
                let n = 1 << nb_bits;
                let s = (0..n).rev();
                test_vec(&mut uart, s.collect());
            }
        }
    }
    println!("uart OK");
}

fn main() {
    test_configs();
}

#[test]
fn run_tests() {
    main()
}
