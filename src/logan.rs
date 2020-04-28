/* Trampoline binary.

   It my current setup, it is simpler to use a single binary to host a
   number of specific parsers.  Note that this is very ad-hoc, and
   currently not very configurable.
*/

extern crate logan;
extern crate derive_more;

use logan::sm::{uart,slip,syncser,diff,apply};
use logan::io::{stdin8,write_byte};
use derive_more::From;

fn start_uart() -> Result<(), AppError>  {
    let mut uart = uart::init(uart::Config {
        period:  samplerate()? / baudrate()?,
        nb_bits: 8,
        channel: 0,
    });
    // uart::test(&mut uart);
    for b in apply(&mut uart, stdin8()) {
        //println!("{}", (b as u8) as char);
        write_byte(b as u8);
    }
    Ok(())

}

fn start_slip() -> Result<(), AppError> {

    let baud = 115200usize;

    let mut slip = slip::init(slip::Config {
        end: 0x0D,
        esc: 0x0C,
        esc_end: 0x0B,
        esc_esc: 0x0A,
    });
    
    let mut uart = uart::init(uart::Config {
        period:  samplerate()? / baud,
        nb_bits: 8,
        channel: 0,
    });
     
    for packet in apply(&mut slip,
                  apply(&mut uart,
                        stdin8())) {
        slip::print(packet);
    }
    Ok(())
}

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

fn start_ice40() -> Result<(), AppError>  {
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
        // logan::io::write_byte(b);
    }
    Ok(())
}

fn start_diff() -> Result<(), AppError>  {
    let mut diff = diff::init();
    for b in apply(&mut diff, stdin8()) {
        println!("{:01$x}",b,2);
        // logan::io::write_byte(b);
    }
    Ok(())
}


fn start() -> Result<(), AppError> {
    let args : Vec<String> = std::env::args().collect() ;
    match &(args[1])[..] {
        "uart"  => start_uart(),
        "slip"  => start_slip(),
        "ice40" => start_ice40(),
        "diff"  => start_diff(),
        _ => Err(AppError::AppStrError("Unknown type"))
    }
}
fn main() {
    match start() {
        Ok(()) => (),
        _ => panic!("Error") // FIXME
    }
}

// To handle multiple errors, put them in an Enum like this
#[derive(From)]
enum AppError {
    AppVarError(std::env::VarError),
    AppParseIntError(std::num::ParseIntError),
    AppStrError(& 'static str)
}

/* Some shared code. */


fn samplerate() -> Result<usize, AppError> { var("LOGAN_SAMPLERATE", 2000000) }
fn baudrate()   -> Result<usize, AppError> { var("LOGAN_BAUDRATE",    115200) }
fn var(varname: &str, default: usize) -> Result<usize, AppError> {
    // let sr_str = std::env::var("LOGAN_SAMPLERATE")?;
    match std::env::var(varname) {
        Ok(sr_str) => {
            let sr = sr_str.parse::<usize>()?;
            Ok(sr)
        },
        Err(_) =>
            Ok(default)
    }
}
