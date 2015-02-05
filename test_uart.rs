
fn test1() {
    let mut uart = la::uart::init(la::uart::Config {
        period:  10,
        nb_bits: 8,
        channel: 0,
    });
    la::uart::test1(&mut uart);
}

extern crate la;
fn main() {
    test1();
}
