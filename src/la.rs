#![feature(io)]
// #![feature(core)]


// A Logic Analyzer is a sequence processor built out of:
//
//   - Tick: run a a rate-reducing state machine for one clock tick:
//     feed in a sample, possibly produce parsed element.
//
//   - Apply: apply the rate-reducer to an arbitrary sequence,
//     collect the result sequence.

pub trait Tick<I,O> {
    fn tick(&mut self, I) -> Option<O>;
}

pub struct Apply<'a,I,S,T:'a,O>
    where S: Iterator<Item=I>, T: Tick<I,O>
{ s: S, t: &'a mut T, }

pub fn apply<I,S,T,O>(tick: &mut T, stream: S) -> Apply<I,S,T,O>
    where S: Iterator<Item=I>, T: Tick<I,O>,
{ Apply { s: stream, t: tick } }

// The inner loop runs until tick produces something, marked (*)
impl<'a,I,S,P,O> Iterator for Apply<'a,I,S,P,O> where
    S: Iterator<Item=I>,
P: Tick<I,O>,
{
    type Item = O;
    fn next(&mut self) -> Option<O> {
        loop { // (*)
            match self.s.next() {
                None => return None,
                Some(input) => match self.t.tick(input) {
                    None => (), // (*)
                    rv => return rv,
                },
            }
        }
    }
}


#[allow(dead_code)]
pub mod diff {
    use Tick;
    #[derive(Copy)]
    pub struct State { last: usize, }
    pub fn init() -> State {State{last: 0}}

    impl Tick<usize,usize> for State {
        fn tick(&mut self, input:usize) -> Option<usize> {
            let x = input ^ self.last;
            self.last  = input;
            if x == 0 { None } else { Some(input) }
        }
    }

}

#[allow(dead_code)]
pub mod uart {

    // Analyzer config and state data structures.
    use Tick;
    use self::Mode::*;
    
    #[derive(Copy)]
    pub struct Config {
        pub period:  usize,    // bit period
        pub nb_bits: usize,
        pub channel: usize,
    }
    pub struct Uart {
        pub config: Config,
        state:  State,
    }
    struct State {
        reg: usize,  // data shift register
        bit: usize,  // bit count
        skip: usize, // skip count to next sample point
        mode: Mode,
    }
    enum Mode {
        Idle, Shift, Break, FrameErr,
    }
    pub fn init(config: Config) -> Uart {
        Uart {
            config: config,
            state: State {
                reg:  0,
                bit:  0,
                skip: 0,
                mode: Idle,
            },
        }
    }

    // Process a single byte, output word when ready.
    impl Tick<usize,usize> for Uart {
        fn tick(&mut self, input :usize) -> Option<usize> {
            let s = &mut self.state;
            let c = &self.config;
            // println!("uart: {} ({} {})", input, s.skip, s.bit);

            if s.skip > 0 {
                s.skip -= 1;
                return None;
            }
            let i = (input >> c.channel) & 1;
            match s.mode {
                Idle => {
                    if i == 0 {
                        s.mode = Shift;
                        s.bit = 0;
                        // Sample halfway in between transitions.
                        // Also valid for period == 1,2.
                        let p1 = c.period - 1;
                        s.skip = p1 + p1 >> 1;
                        s.reg = 0;
                    }
                    return None;
                },
                Shift => {
                    // data bit
                    if s.bit < c.nb_bits {
                        s.reg |= i << s.bit;
                        s.bit += 1;
                        s.skip = c.period - 1;
                        return None;
                    }
                    // stop bit
                    else {
                        s.skip = 0;
                        if i == 1 {
                            s.mode = Idle;
                            return Some(s.reg);
                        }
                        else {
                            s.mode = match s.reg {
                                0 => Break,
                                _ => FrameErr,
                            };
                            return None;
                        }
                    }
                },
                // FIXME: Break and FrameErr will auto-recover.
                // Not necessarily what you want.
                _ => {
                    if i == 1 { s.mode = Idle; }
                    return None;
                }
            }
        }
    }
}

pub mod io {
    use std::old_io;

    /* Manually buffered standard input.  Buffer size such that write from
    Saleae driver doesn't need to be chunked. */
    pub struct Stdin8 {
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


