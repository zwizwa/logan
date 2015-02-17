extern crate la;
use la::decode;
use la::slip::{Config,init};


fn test1() {
    let mut slip = init(Config{
        end: 0x0D,
        esc: 0x0C,
        esc_end: 0x0B,
        esc_esc: 0x0A,
    });
    for v in decode(&mut slip, [0x0D,1,2,3,0x0D,4,5,6,0xD].iter()) {
        la::slip::print(v);
    }
    println!("slip OK");
}

fn main() {
    test1();
}

#[test]
fn run_tests() {
    main()
}
