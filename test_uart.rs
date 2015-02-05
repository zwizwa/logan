extern crate la;

fn frame(nb_bits: usize, value: usize) -> usize {
    (value | (1 << nb_bits)) << 1
}

fn test_vec(uart: &mut la::uart::Env, data_in: Vec<usize>) {
    let nb_bits = uart.config.nb_bits;
    let period  = uart.config.period;
    let data_out: Vec<_> =
        la::apply(uart,
                  // expand data to bits to samples.
                  data_in.iter()
                  .flat_map(|&data| (0..nb_bits+2).map(move |shift| (frame(nb_bits, data) >> shift) & 1))
                  .flat_map(|bit|   (0..period).map(move |_| bit))
                  ).collect();
    assert_eq!(data_out, data_in);
    println!("test1 OK");
}


fn test1() {
    let mut uart = la::uart::init(la::uart::Config {
        period:  3,
        nb_bits: 8,
        channel: 0,
    });
    test_vec(&mut uart, (0..256).collect());
}

fn main() {
    test1();
}
