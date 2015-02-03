#![feature(io)]
#![feature(core)]

extern crate core;

mod la {
    /* A Logic Analyzer is built out of:
       - Proc: a rate-reducing state machine: feed in an element, produce 0 or more results.
       - ProcMap: apply the rate-reducer over an arbitrary sequence, collect the result sequence. */

    pub trait Proc<I,O> {
        fn tick(&mut self, I) -> Option<O>;
    }

    // Boiler plate: iterator state and public constructor.
    struct ProcMap<I,S,P,O>
        where S: Iterator<Item=I>, P: Proc<I,O>
    { s: S, p: P, }
    
    pub fn proc_map<I,S,P,O>(p: P, s: S) -> ProcMap<I,S,P,O>
        where S: Iterator<Item=I>, P: Proc<I,O>,
    { ProcMap { s: s, p: p } }

    // Functionality is in the trait implementation.
    impl<I,S,P,O> Iterator for ProcMap<I,S,P,O> where
        S: Iterator<Item=I>,
        P: Proc<I,O>,
    {
        type Item = O;
        fn next(&mut self) -> Option<O> {
            loop {
                match self.s.next() {
                    None => return None,
                    Some(input) => match self.p.tick(input) {
                        None => (),
                        rv => return rv,
                    },
                }
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
    #[inline]
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
    struct Stream<'a, I: Iterator<Item=usize>> {
        uart: Env,
        iter: I,
    }
    // TODO: generalize "trickle" map over state machine.  None,None,Some,None,....
    impl<'a,I> Iterator for Stream<'a,I> where
        I: Iterator<Item=usize>,
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
        I:Iterator<Item=usize>,
    {
        Stream {
            uart: init(),
            iter: i,
        }
    }

    // TODO: use FlatMap

    
    #[allow(dead_code)]

    pub fn test(uart : &mut Env) {

        // FIXME: get this nicer version to work.  Doesn't specialize
        // properly + some lifetime issues for the closures.
        
        // let period = uart.config.period;
        // let word = uart.config.nb_bits;

        // let bits_bit   = |v| (0..period).map(|_| v);
        // let bits_frame = |v| (0..word+2).map(|bit| (((v | (1 << word)) << 1) >> bit) & 1);

        // let samples  =
        //     (0..256)
        //     .flat_map(bits_frame)
        //     .flat_map(bits_bit);

        // for b in stream(samples) {
        //     println!("data {}", b);
        // }

        for data in 0us..256 {
            let bits = (data | 0x100) << 1; // add start, stop bit
            for i in 0us..(uart.config.nb_bits+2) {
                let bit = ((bits >> i) & 1) << uart.config.channel;
                for _ in 0..uart.config.period {
                    match tick(uart, bit) {
                        None => (),
                        Some(out_data) => 
                            if out_data != data {
                                panic!("out_data:{} != in_data:{}", out_data, data)
                            }
                    }
                }
            }
        }
        println!("Test OK");
    }
}

mod io {
    use std::old_io;

    /* Manually buffered standard input.  Buffer size such that write from
    Saleae driver doesn't need to be chunked. */
    struct Stdin8 {
        stream: old_io::stdio::StdinReader,
        buf: [u8; 262144],
        offset: usize, // FIXME: couldn't figure out how to use slices.
        nb: usize,
    }
    impl Iterator for Stdin8 {
        type Item = usize;
        fn next(&mut self) -> Option<usize> {
            loop {
                let o = self.offset;
                if o < self.nb {
                    let rv = self.buf[o];
                    self.offset += 1;
                    return Some(rv as usize);
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
    pub fn stdin8<'a>() -> Stdin8 {
        Stdin8 {
            stream: old_io::stdin(),
            buf: [0u8; 262144],
            offset: 0,
            nb: 0,
        }
    }
}

fn main() {
    let mut uart = uart::init();
    uart.config.channel = 3;
    uart::test(&mut uart);

    for b in uart::stream(io::stdin8()) {
        println!("data {}", b);
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
