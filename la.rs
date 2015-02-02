#![feature(io)]

mod la {
    use std::old_io as io;

    //pub type Chunk = &Iterator<Item=&u8>;   // FIXME: lifetime specifier?
    pub type Chunk = [u8]; // FIXME: use this until iterators are fixed in rust


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
        fn push(&mut self, &Chunk);

        fn stdin_u8(&mut self, buf: &mut [u8]) {
            let mut i = io::stdin();
            loop {
                match i.read(buf) {
                    Err(why) => panic!("{:?}", why),
                    Ok(size) => Sink::push(self, &buf[0..size]),
                }
            }
        }
    }
}


// Implement stdin as an iterator of buffers.
struct Stdin<'a> {
    stream: std::old_io::stdio::StdReader,
    buf: &'a mut[u8],
}
impl<'a> Iterator for Stdin<'a> {
    type Item = &'a[u8];
    fn next(&'a mut self) -> Option<Iterator::Item> {
        match self.stream.read(self.buf) {
            Err(why) => None,
            Ok(size) => Some(self.buf),
        }
    }
}
    




#[allow(dead_code)]
mod diff {
    struct Diff {
        last: usize,
    }
    pub fn tick(diff: &mut Diff, input: usize) {
        let x = input ^ diff.last;
        diff.last = input;
        println!("diff: {}", x);
    }
}

mod uart {
    use la::{Sink};
    
    use self::Mode::*;
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

    impl Sink for Uart {
        fn push(&mut self, input: &[u8]) {
            for byte in input.iter() {
                tick(self, (*byte) as usize);
            }
        }
    }

    fn tick (uart : &mut Uart, bus : usize) {
        let s = &mut uart.state;
        let c = &uart.config;

        if s.skip > 0 {
            s.skip -= 1;
        }
        else {
            let i = bus >> c.channel;
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
    pub fn test(uart : &mut Uart) {
        for data in 0us..256 {
            let bits = (data | 0x100) << 1; // add start, stop bit
            for i in 0us..(uart.config.nb_bits+2) {
                let b = ((bits >> i) & 1) << uart.config.channel;
                for _ in 0..uart.config.period { tick(uart, b); };
            }
            if uart.state.reg != data {
                panic!("reg:{} != data:{}", uart.state.reg, data);
            }
        }
    }
}
fn main() {
    let mut uart = uart::init();
    uart.config.channel = 3;
    uart::test(&mut uart);
    la::Sink::stdin_u8(&mut uart, &mut [0u8; 1024 * 256]);
}
