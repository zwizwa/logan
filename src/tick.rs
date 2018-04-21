//#![feature(io)]
//#![feature(core)]


// A Logic Analyzer is a sequence processor built out of:
//
//   - Tick: run a a state machine for one clock tick, feeding it a
//     parallel logic sample, possibly producing a parsed element.
//
//   - Apply: apply the rate-reducer to a parallel logic sequence,
//     collect the result sequence.


// ---- Tick ----

pub trait Tick<I,O> {
    fn tick(&mut self, I) -> Option<O>;
}
pub trait Bus {
    fn channel(&self, usize) -> usize;
    fn as_usize(&self) -> usize;
}


macro_rules! impl_Bus {
    ($t:ty) => (
        impl Bus for $t {
            #[inline(always)]
            fn channel(&self, c:usize) -> usize {
                (((*self) >> c) & 1 ) as usize
            }
            #[inline(always)]
            fn as_usize(&self) -> usize {
                (*self) as usize
            }
        });
    }
impl_Bus!(u8);
impl_Bus!(usize);
impl_Bus!(i32);

impl<'a,T> Bus for &'a T where T: Bus {
    #[inline(always)]
    fn channel(&self, c:usize) -> usize { (*self).channel(c) }
    fn as_usize(&self) -> usize { (*self).as_usize() }
}
    

pub mod diff {
    use tick::Tick;
    use tick::Bus;
    #[derive(Copy,Clone)]
    pub struct State { last: usize, }
    pub fn init() -> State {State{last: 0}}

    impl<B> Tick<B,usize> for State where B: Bus {
        #[inline(always)]
        fn tick(&mut self, input_bus:B) -> Option<usize> {
            let input = input_bus.as_usize();
            let x = input ^ self.last;
            self.last  = input;
            if x == 0 { None } else { Some(input) }
        }
    }
}

pub mod uart {

    // Analyzer config and state data structures.
    use tick::Tick;
    use self::Mode::*;
    
    #[derive(Copy,Clone)]
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
        clocks: usize,
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
                clocks: 0,
            },
        }
    }

    #[inline(always)]
    pub fn start_delay(period: usize) -> usize {
        // Picking initial delay after start seems tricky.  What would
        // be a good theory?  This formula works for synthetic signals
        // period 1 to 10 and tested IRL.  Theory?
        let p1 = period - 1;
        p1 + (p1 >> 1) + (period >> 2)
    }

    // Process a single byte, output word when ready.
    impl<B> Tick<B,usize> for Uart where B: super::Bus {
        #[inline(always)]
        fn tick(&mut self, input :B) -> Option<usize> {
            let s = &mut self.state;
            let c = &self.config;

            s.clocks += 1;

            if s.skip > 0 {
                s.skip -= 1;
                return None;
            }
            let i = input.channel(c.channel); // ^ 0x1234567800000000; // dasm marker
            // println!("uart: {:x} ({} {} {})", input.as_usize(), s.skip, s.bit, s.clocks);
            match s.mode {
                Idle => {
                    if i == 0 {
                        s.mode = Shift;
                        s.bit = 0;
                        s.skip = start_delay(c.period);
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
                            println!("FrameErr/Break 0x{:x}", s.reg);
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


pub mod syncser {
    // transliterated from pyla/syncser.cpp

    // (A) Is it necessary to provide a LSBit first shift?  Both SPI
    //     and I2C seem to use MSBit first in all cases I've
    //     encountered.
    //
    // (B) For word-oriented streams, it might be good to shift in
    //     full words, then allow endianness config in the output
    //     stream.
   
    use tick::Tick;

    /* SPI clock configurations can be confusing as there are many
    ways to express the same information.  Thus uses the following
    convention:

    - clock_edge: 
      0  sample on 1->0 transition
      1  sample on 0->1 transition

    - clock_polarity:
      0  clock starts out at 0 level
      1  clock starts out at 1 level

    =>
    phase = 0  (sample on first edge)  when clock_edge != clock_polarity
    phase = 1  (sample on second edge) when clock_edge == clock_polarity

    https://en.wikipedia.org/wiki/Serial_Peripheral_Interface_Bus#Mode_numbers
    
    */
    
    #[derive(Copy,Clone)]
    pub struct Config {
        pub clock_channel:  usize,
        pub data_channel:   usize,
        pub frame_channel:  usize,   // chip select
        pub clock_edge:     usize,
        pub clock_polarity: usize,
        pub frame_active:   usize,
        pub frame_timeout:  usize,
        pub nb_bits:        usize,
        pub frame_enable:   bool,
        pub timeout_enable: bool,
    }
    struct State {
        clock_state: usize,
        frame_state: usize,
        shift_count: usize,
        shift_reg: usize,
        frame_timeout_state: usize,
    }
    pub struct SyncSer {
        pub config: Config,
        state: State,
    }
    
    pub fn config() -> Config {
        Config {
            clock_channel: 0,
            data_channel:  1,
            frame_channel: 0,
            frame_enable: false,
            clock_edge: 1,     // positive edge triggering
            clock_polarity: 0, // idle clock
            frame_active: 0,
            frame_timeout: 0, // disabled
            timeout_enable: false,
            nb_bits: 8,
        }
    }
    pub fn init(c: Config) -> SyncSer {
        SyncSer {
            config: c,
            state: State {
                clock_state: c.clock_polarity,
                frame_state: c.frame_active ^ !1,
                frame_timeout_state: 0,
                shift_count: 0,
                shift_reg: 0,
            }
        }
    }

    impl<B> Tick<B,usize> for SyncSer where B: super::Bus {
        #[inline(always)]
        fn tick(&mut self, input :B) -> Option<usize> {   

            let s = &mut self.state;
            let c = &self.config;

            let clock_bit = input.channel(c.clock_channel);
            let frame_bit = input.channel(c.frame_channel);
            let data_bit  = input.channel(c.data_channel);

            let mut rv = None;

            // Frame edge
            // FIXME: this should wait to do anything if it starts in the
            // middle of a frame.
            if c.frame_enable {
                if frame_bit != s.frame_state { // transition
                    if frame_bit == c.frame_active {
                        // reset shift register
                        s.shift_reg = 0;
                        s.shift_count = 0;
                    }
                }
            }
            
            // Frame timeout.
            if c.timeout_enable {
                if c.frame_timeout > 0 {
                    if s.frame_timeout_state == 0 {
                        // reset
                        s.shift_reg = 0;
                        s.shift_count = 0;
                        s.frame_timeout_state = c.frame_timeout;
                    }
                    else {
                        s.frame_timeout_state -= 1;
                    }
                }
            }

            // Shift in data on sampling clock edge.
            if !c.frame_enable || (frame_bit == c.frame_active) {
                if clock_bit != s.clock_state {  // transition
                    if clock_bit == c.clock_edge { // sampling edge
                        s.shift_reg <<= 1; // (A) 
                        s.shift_reg |= data_bit;
                        s.shift_count += 1;
                        if s.shift_count == c.nb_bits { // (B)
                            rv = Some(s.shift_reg);
                            // reset shift register
                            s.shift_reg = 0;
                            s.shift_count = 0;
                            // reset frame timeout
                            s.frame_timeout_state = c.frame_timeout;
                        }
                    }
                }
            }

            // Edge detector state
            s.clock_state = clock_bit;
            s.frame_state = frame_bit;

            return rv;
        }
    }
}
pub mod slip {
    use tick::Tick;
    use tick::Bus;
    use std::mem;
    
    #[derive(Copy,Clone)]
    pub struct Config {
        pub end:     u8,
        pub esc:     u8,
        pub esc_end: u8,
        pub esc_esc: u8,
    }
    pub struct State {
        buf: Vec<u8>,
        esc: bool,
    }
    pub struct Slip {
        config: Config,
        state:  State,
    }
    pub fn init(c: Config) -> Slip {
        Slip {
            config: c,
            state: State {
                buf: Vec::new(),
                esc: false,
            }
        }
    }
    impl<B> Tick<B,Vec<u8>> for Slip where B: Bus {
        #[inline(always)]
        fn tick(&mut self, input_bus:B) -> Option<Vec<u8>> {
            let c = &self.config;
            let s = &mut self.state;
            let i = input_bus.as_usize() as u8;
            if s.esc {
                s.esc = false;
                if      c.esc_end == i { s.buf.push(c.end); }
                else if c.esc_esc == i { s.buf.push(c.esc); }
                return None;
            }
            if c.esc == i {
                s.esc = true;
                return None;
            }
            if c.end == i {
                let packet = mem::replace(&mut s.buf, Vec::new());
                return Some(packet);
            }
            s.buf.push(i);
            return None;
        }
    }
    pub fn print(v: Vec<u8>) {
        print!("({}) -", v.len());
        for e in v { print!(" {:01$x}", e, 2); }
        println!("");
    }
}





// ---- Apply ----

pub struct Apply<'a,I:'a,O:'a> {
    s: &'a mut Iterator<Item=I>,
    t: &'a mut Tick<I,O>,
}

pub fn apply<'a,I:'a,O:'a>
    (tick:   &'a mut Tick<I,O>,
     stream: &'a mut Iterator<Item=I>) -> Apply<'a,I,O>  {
    Apply { s: stream, t: tick }
}

impl<'a,I,O> Iterator for Apply<'a,I,O> where
{
    type Item = O;
    #[inline(always)]
    fn next(&mut self) -> Option<O> {
        loop {
            match self.s.next() {
                None => return None,
                Some(input) => match self.t.tick(input) {
                    None => (),
                    rv => return rv,
                },
            }
        }
    }
}

