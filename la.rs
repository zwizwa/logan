#![feature(io)]
use std::old_io as io;
    
mod uart {
    use self::Mode::*;
    struct Uart {
        pub config: Config,
        state:  State,
    }
    struct Config {
        pub period:  u32,    // bit period
        pub nb_bits: u32,
        pub channel: u32,
    }
    //#[derive(Debug)]
    struct State {
        reg: u32,  // data shift register
        bit: u32,  // bit count
        skip: u32, // skip count to next sample point
        mode: Mode,
    }
    //#[derive(Debug)]
    enum Mode {
        Idle, Shift, Stop,
    }
    pub fn init() -> Uart {
        Uart {
            config: Config {
                period:  1000,
                nb_bits: 8,
                channel: 0,
            },
            state: State {
                reg:  0,
                bit:  0,
                skip: 0,
                mode: Idle,
            },
        }
    }

    pub fn tick (uart : &mut Uart, input : u32) {
        let s = &mut uart.state;
        let c = &uart.config;

        if s.skip > 0 {
            s.skip -= 1;
        }
        else {
            //println!("{:?}", s.mode);
            let i = (input >> c.channel) & 1;
            match s.mode {
                Idle => {
                    if i == 0 {
                        s.mode = Shift;
                        s.bit = 0;
                        s.skip = c.period / 2;
                    }
                },
                Shift => {
                    s.reg = (s.reg << 1) | i;
                    s.bit += 1;
                    s.skip = c.period;
                    if s.bit > c.nb_bits {
                        s.mode = Stop;
                    }
                },
                Stop => {
                    if i == 0 { panic!("frame error"); }
                    println!("data: {}", s.reg);
                    s.skip = c.period / 2;
                    s.mode = Idle;
                    s.reg = 0;
                },
            }
        }
    }
    pub fn frontend(state : &mut Uart, raw : &[u8]) {
        // println!("size: {}", raw.len());
        for i in raw.iter() {
            tick(state, (*i) as u32);
        }
    }
    #[allow(dead_code)]
    pub fn test() {
        let mut state = init();
        state.config.period = 10;
        let data : u32 = 0xFFFFFF7F;
        for i in 0u8..32 {
            let b = (data >> i) & 1;
            let v = [b as u8; 10];
            frontend(&mut state, &v);
        }
    }
}
fn main() {
    uart::test();
    let mut state = uart::init();
    state.config.channel = 1;
    let mut i = io::stdin();
    loop {
        let buf = &mut [0u8; 1024 * 256];
        match i.read(buf) {
            Err(why) => panic!("{:?}", why),
            Ok(size) => uart::frontend(&mut state, &buf[0 .. size]),
        }
    }
}
