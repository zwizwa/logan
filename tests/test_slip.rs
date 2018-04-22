extern crate la;
use la::sm::apply;
use la::sm::slip;


fn test1() {
    let mut slip = slip::init(
        slip::Config{
            end: 0x0D,
            esc: 0x0C,
            esc_end: 0x0B,
            esc_esc: 0x0A,
        }
    );
    let mut test_data = [0x0D,1,2,3,0x0D,4,5,6,0xD].iter();
    let data_out: Vec<_> = apply(&mut slip, &mut test_data).collect();

    let data_expected: Vec<Vec<u8>> = [
        [].to_vec(),
        [1,2,3].to_vec(),
        [4,5,6].to_vec()
    ].to_vec();

    assert_eq!(data_out, data_expected);

    println!("slip OK");
}

fn main() {
    test1();
}

#[test]
fn run_tests() {
    main()
}
