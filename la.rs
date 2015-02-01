#![feature(io)]

mod la {

    pub type Bit = usize;
    pub type Bus = usize;

    pub fn channel(b: Bus, c: usize) -> Bit {
        (b >> c) & 1
    }

    // TL;DR: Call flow represents low level call flow.  High level
    // abstractions build on top of this.

    // From a first iteration in C++ (see zwizwa/pyla), the simplest
    // architecture seems to be a "push" approach, i.e. a data-driven
    // / reactive one where a function call corresponds to data being
    // available.

    // This corresponds best to the actual low-level structure when
    // this runs on a uC: a DMA transfer_complete interrupt.

    // It is opposed to a "pull" approach where a task blocks until
    // data is available, which always needs a scheduler.  The pull
    // approach works best on higher levels, i.e. when parsing
    // protocols.

    pub trait Sink {
        fn write(&mut self, &[Bus]);
    }
}



#[allow(dead_code)]
mod diff {
    use la::Bus;
    struct Diff {
        last: Bus,
    }
    pub fn tick(diff: &mut Diff, input: Bus) {
        let x = input ^ diff.last;
        diff.last = input;
        println!("diff: {}", x);
    }
}

mod uart {
    use la::{Bus,channel};
    
    use self::Mode::*;
    use std::old_io as io;
    struct Uart {
        pub config: Config,
        state:  State,
    }
    struct Config {
        pub period:  usize,    // bit period
        pub nb_bits: usize,
        pub channel: usize,
    }
    //#[derive(Debug)]
    struct State {
        reg: usize,  // data shift register
        bit: usize,  // bit count
        skip: usize, // skip count to next sample point
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

    pub fn tick (uart : &mut Uart, input : Bus) {
        let s = &mut uart.state;
        let c = &uart.config;

        if s.skip > 0 {
            s.skip -= 1;
        }
        else {
            // println!("{:?}", s.mode);
            let i = channel(input, c.channel);
            match s.mode {
                Idle => {
                    if i == 0 {
                        s.mode = Shift;
                        s.bit = 0;
                        s.skip = c.period + (c.period / 2) - 1;
                        s.reg = 0;
                    }
                },
                Shift => {
                    if s.bit < c.nb_bits {
                        s.reg |= i << s.bit;
                        s.bit += 1;
                        s.skip = c.period - 1;
                    }
                    else {
                        s.mode = Stop;
                    }
                },
                Stop => {
                    if i == 0 { panic!("frame error"); }
                    println!("uart: {}", s.reg);
                    s.skip = 0;
                    s.mode = Idle;
                },
            }
        }
    }
    #[allow(dead_code)]
    pub fn run_stdin() {
        let mut uart = init();
        uart.config.channel = 1;
        let mut i = io::stdin();
        loop {
            let buf = &mut [0u8; 1024 * 256];
            match i.read(buf) {
                Err(why) => panic!("{:?}", why),
                Ok(size) => for b in buf[0..size].iter() { tick(&mut uart, (*b) as Bus); },
            }
        }
    }
    
    #[allow(dead_code)]
    pub fn test(uart : &mut Uart) {
        uart.config.period = 10;
        for data in 0us..256 {
            let bits = (data | 0x100) << 1; // add start, stop bit
            for i in 0us..(uart.config.nb_bits+2) {
                let b = (bits >> i) & 1;
                for _ in 0..uart.config.period { tick(uart, b); };
            }
            if uart.state.reg != data {
                panic!("reg:{} != data:{}", uart.state.reg, data);
            }
        }
    }
}
fn main() {
    uart::test(&mut uart::init());
    // uart::run_stdin();
}
