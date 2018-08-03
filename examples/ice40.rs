/* Illustrating SPI for a slightly more involved example: booting a
   iCE40 FPGA.  This involves multiple signals.

   GND                      (black)

   BBB SPI0 test
   0
   1
   2 CDONE  gpio3_14 P9_31
   3 CRESET gpio3_15 P9_29  (green)
   4 MOSI (D1)       P9_18  (blue)
   5 MISO (D0)       P9_21  (purple) 
   6 SCLK            P9_22  (grey)
   7 CS     gpio1_12 P8_12  (white)
*/

#![feature(core)]
use la::sm::{apply,syncser};
use la::io::stdin8;
extern crate la;
fn main() {
    let config = syncser::Config {
        clock_channel:   5,
        data_channel:    4,
        frame_channel:   0,
        clock_edge:      0,
        clock_polarity:  0,
        frame_enable:    true,
        frame_active:    0,
        frame_timeout:   0, //disabled
        timeout_enable:  false,
        nb_bits:         8
    };
    let mut syncser = syncser::init(config);
    for b in apply(&mut syncser, stdin8()) {
        println!("{:01$x}",b,2);
        // la::io::write_byte(b);
    }
}
