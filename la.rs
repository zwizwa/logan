#![feature(io)]
#![feature(core)]

extern crate core;



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

    // Analyzer config and state data structures.
    use self::Mode::*;
    pub struct Env {
        pub config: Config,
        state:  State,
    }
    struct Config {
        pub period:  usize,    // bit period
        pub nb_bits: usize,
        pub channel: usize,
    }
    struct State {
        reg: usize,  // data shift register
        bit: usize,  // bit count
        skip: usize, // skip count to next sample point
        mode: Mode,
    }
    enum Mode {
        Idle, Shift, Stop,
    }
    pub fn init() -> Env {
        Env {
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

    // Process a single byte, output word when ready.
    fn tick (uart: &mut Env, input: usize) -> Option<usize>  {
        let s = &mut uart.state;
        let c = &uart.config;

        let mut rv = None;

        if s.skip > 0 {
            s.skip -= 1;
        }
        else {
            let i = input >> c.channel;
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
                    rv = Some(s.reg);
                    s.skip = 0;
                    s.mode = Idle;
                },
            }
        }
        rv
    }

    // Export behavior as iterator.
    struct Stream<'a, I: Iterator<Item=u8>> {
        uart: Env,
        iter: I,
    }
    impl<'a,I> Iterator for Stream<'a,I> where
        I: Iterator<Item=u8>,
    {
        type Item = usize;
        fn next(&mut self) -> Option<usize> {
            loop {
                match self.iter.next() {
                    None => return None,
                    Some(bit) => match tick(&mut self.uart, bit as usize) {
                        None => (),
                        rv => return rv,
                    },
                }
            }
        }
    }
    pub fn stream<'a,I>(i: I) -> Stream<'a,I> where
        I:Iterator<Item=u8>,
    {
        Stream {
            uart: init(),
            iter: i,
        }
    }

    // TODO: use FlatMap

    
    #[allow(dead_code)]
    pub fn test(uart : &mut Env) {
        for data in 0us..256 {
            // let check_data = |&:data_out : usize| {
            //     if data_out != data {
            //         panic!("check_data: {} != {}", data_out, data);
            //     }
            // };
            let bits = (data | 0x100) << 1; // add start, stop bit
            for i in 0us..(uart.config.nb_bits+2) {
                let bit = ((bits >> i) & 1) << uart.config.channel;
                for b in 0..uart.config.period {
                    match tick(uart, b) {
                        None => (),
                        Some(out_data) => 
                            if out_data != data {
                                panic!("reg:{} != data:{}", out_data, data)
                            },
                    }
                }
            }
        }
    }
}
fn main() {
    let mut uart = uart::init();
    uart.config.channel = 3;
    uart::test(&mut uart);

    let buf = [0u8; 262144];
}

mod io {
    use std::old_io;

    /* Manually buffered standard input.  Buffer size such that write from
    Saleae driver doesn't need to be chunked. */
    struct Stdin {
        stream: old_io::stdio::StdinReader,
        buf: [u8; 262144],
        offset: usize, // FIXME: couldn't figure out how to use slices.
        nb: usize,
    }
    impl Iterator for Stdin {
        type Item = u8;
        fn next(&mut self) -> Option<u8> {
            loop {
                let o = self.offset;
                if o < self.nb {
                    let rv = self.buf[o];
                    self.offset += 1;
                    return Some(rv);
                }
                match self.stream.read(&mut self.buf) {
                    Err(_) => return None,
                    Ok(nb) => {
                        self.offset = 0;
                        self.nb = nb;
                    }
                }
            }
        }
    }
    pub fn stdin<'a>() -> Stdin {
        Stdin {
            stream: old_io::stdin(),
            buf: [0u8; 262144],
            offset: 0,
            nb: 0,
        }
    }
}




    
// // Expose uart as a sequence of bytes
// struct Uart<I> {
//     env: uart::Env,
// }

// impl<'b> Iterator for Uart {
//     type Item = usize;
//     fn next<'a>(&'a mut self) -> Option<usize> {
//         match self.stream.read(&mut self.buf) {
//             Err(_) => None,
//             Ok(size) => Some(&self.buf[0..size]),
//         }
//     }
// }
