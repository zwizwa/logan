extern crate la;

fn frame(nb_bits: usize, value: usize) -> usize {
    (value | (1 << nb_bits)) << 1
}
fn test_vec(uart: &mut la::uart::Uart, data_in: Vec<usize>) {
    let c = uart.config;
    let data_out: Vec<_> =
        la::apply(uart,
                  data_in.iter()
                  // expand data word to UART frame bit sequence
                  .flat_map(|&data| (0..c.nb_bits+2).map(move |shift| (frame(c.nb_bits, data) >> shift) & 1))
                  // shift it to the correct channel on the bus
                  .map(|bit| bit << c.channel)
                  // oversample bit sequence
                  .flat_map(|bus|   (0..c.period).map(move |_| bus))
                  ).collect();
    assert_eq!(data_out, data_in);
    println!("test1 OK");
}


fn test1() {
    let mut uart = la::uart::init(la::uart::Config {
        period:  10,
        nb_bits: 8,
        channel: 3,
    });
    test_vec(&mut uart, (0..256).collect());
}

fn main() {
    test1();
}
