#![feature(io)]

use std::old_io as io;
extern crate core;

mod la {
    //pub type Input = &Iterator<Item=&u8>;   // FIXME: lifetime specifier?
    pub type Input = [u8]; // FIXME: use this until iterators are fixed in rust
    pub type Output<'a> = FnMut(usize)+'a;

    pub trait Sink {
        fn process(&mut self, &Input, Output);
    }
}

pub struct UartProc<'a, I: Iterator<Item=&'a [u8]>> {
    uart: uart::Env,
    iter: I,
    biter: core::slice::Iter<'a, u8>,
}

impl<'a,I> Iterator for UartProc<'a,I> where
    I: Iterator<Item=&'a [u8]>,
{
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        loop {
            match self.biter.next() {
                None => match self.iter.next() {
                    None => return None,
                    Some(bs) => self.biter = bs.iter(),
                },
                Some(b) => return Some((*b) as usize),
            }
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
    use la::{Sink,Input,Output};
    
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
    
    pub fn process(uart: &mut Env, input: &Input) {
        for byte in input.iter() {
            tick(uart, (*byte) as usize);
        }
    }

    fn tick (uart: &mut Env, input: usize)  {
        let s = &mut uart.state;
        let c = &uart.config;

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
                    // output(s.reg);
                    println!("data {}", s.reg);
                    s.skip = 0;
                    s.mode = Idle;
                },
            }
        }
    }


    
    #[allow(dead_code)]
    pub fn test(uart : &mut Env) {
        let mut buf = [0u8; 100];
        uart.config.period = buf.len();
        for data in 0us..256 {
            // let check_data = |&:data_out : usize| {
            //     if data_out != data {
            //         panic!("check_data: {} != {}", data_out, data);
            //     }
            // };
            let bits = (data | 0x100) << 1; // add start, stop bit
            for i in 0us..(uart.config.nb_bits+2) {
                let bit = ((bits >> i) & 1) << uart.config.channel;
                for b in buf.iter_mut() { *b = bit as u8 };
                process(uart, &buf);
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

    let mut stdin = Stdin {
        stream: io::stdin(),
        buf:[0u8; 262144],
    };
    fn data(b: usize) { println!("data: {}",b); }

    for buf in stdin {
        uart::process(&mut uart, buf);
    }
}

// Expose stdin as a sequence of buffers.
struct Stdin {
    stream: std::old_io::stdio::StdinReader,
    buf: [u8; 262144],
}
impl<'b> Iterator for Stdin {
    type Item = &'b[u8];
    fn next<'a>(&'a mut self) -> Option<&'a [u8]> {
        match self.stream.read(&mut self.buf) {
            Err(_) => None,
            Ok(size) => Some(&self.buf[0..size]),
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
