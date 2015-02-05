
fn test1() {
    let mut uart = la::uart::init(la::uart::Config {
        period:  3,
        nb_bits: 8,
        channel: 0,
    });
    la::uart::test1(&mut uart, (0..256).collect());

}

extern crate la;
fn main() {
    test1();
}
