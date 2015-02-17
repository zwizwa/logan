#![feature(io)]

// A Logic Analyzer is a sequence processor built out of:
//
//   - Tick: run a a state machine for one clock tick, feeding it a
//     parallel logic sample, possibly producing a parsed element.
//
//   - Decode: apply the rate-reducer to a parallel logic sequence,
//     collect the result sequence.

pub trait Tick<I,O> {
    fn tick(&mut self, I) -> Option<O>;
}
pub trait Bus {
    fn channel(&self, usize) -> usize;
    fn as_usize(&self) -> usize;
}

pub struct Decode<'a,I,S,T:'a,O>
    where S: Iterator<Item=I>, T: Tick<I,O>
{ s: S, t: &'a mut T, }

pub fn decode<I,S,T,O>(tick: &mut T, stream: S) -> Decode<I,S,T,O>
    where S: Iterator<Item=I>, T: Tick<I,O>,
{ Decode { s: stream, t: tick } }

impl<'a,I,S,P,O> Iterator for Decode<'a,I,S,P,O> where
    S: Iterator<Item=I>,
P: Tick<I,O>,
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
    

pub mod diff {
    use Tick;
    use Bus;
    #[derive(Copy)]
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


pub mod syncser {
    // transliterated from pyla/syncser.cpp

    // (A) Is it necessary to provide a LSBit first shift?  Both SPI
    //     and I2C seem to use MSBit first in all cases I've
    //     encountered.
    //
    // (B) For word-oriented streams, it might be good to shift in
    //     full words, then allow endianness config in the output
    //     stream.
   
    use Tick;

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
    
    #[derive(Copy)]
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
                frame_state: c.frame_active ^ -1,
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
    use Tick;
    use Bus;
    
    #[derive(Copy)]
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
                let rv = s.buf;
                s.buf = Vec::new();
                //return None;
                return Some(rv);
            }
            s.buf.push(i);
            return None;
        }
    }
}

pub mod mipmap {

    /* mipmap storage for fast GUI zoom.

    Starting from 2^N block size, a mipmap store is twice as large.
    Each channel has a min and max stored (and an or).  Transliterated
    from pyla/doodle/mipmap.cpp

    It might be useful to only store coarse levels skipping p fine
    levels, reducing storage to only an extra faction 1/2^p

    */


    /* Representation

    Each channel is two bits: [max|min]
    
    01 == low
    10 == high
    11 == both
    00 == neither

    That encoding allows bitwize-or to compute next mipmap.
    
    Buffer can be initialized from two complementary bit frames.
    
    */

    pub trait MipMap {
        fn plane_init(&self) -> (Self, Self);      // orig -> level 0
        fn plane_or(&self, other: &Self) -> Self;  // level n -> level n + 1
    }
    // Is there a trait to abstract logic operations instead of this workaround?
    macro_rules! impl_MipMap {
        ($t:ty) => (
            impl MipMap for $t {
                fn plane_init(&self) -> ($t,$t) {
                    let bit1 = *self;
                    let bit0 = bit1 ^ -1;
                    (bit0, bit1)
                }
                fn plane_or(&self, other: &$t) -> $t {
                    (*self) | (*other)
                }
            });
    }
    impl_MipMap!(usize);
    impl_MipMap!(u8);
    impl_MipMap!(u16);
    impl_MipMap!(u32);
    impl_MipMap!(u64);
    
    
    #[inline(always)]
    fn log2_upper(x: usize) -> usize {
        let mut v = x - 1;
        let mut log = 0;
        while v != 0  {
            log+=1;
            v>>=1;
        }
        log
    }

    /* Using a flat power-of-two buffer makes data access to all
    levels very simple: the pattern in the address bits can be used to
    access levels directly using a signed int shift trick.

    level 0 = original
    level n = nth reduction

    The address of the first element of each level looks like this:

    0.....  level 1
    10....  level 2
    110...  etc..
    1110..
    11110.  

    Where the dots carry the high bits of the original address.

    Now build that pattern using shifts & masks.

    */

    #[inline(always)]
    fn level_offset(index: usize, level: usize, nb_levels: usize) -> usize {
        let mask       = (1 << nb_levels) - 1;
        let left_ones  = (-1) << (nb_levels - level + 1);
        let trunc_addr = (index & mask) >> level;
        let addr       = (left_ones | trunc_addr) & mask;
        addr
    }

    #[inline(always)]
    fn level_o_n(level: usize, nb_levels: usize) -> (usize, usize) {
        let o = level_offset(0, level, nb_levels);
        let n = 1 << (nb_levels - level);
        println!("{} {:x} {}", level, o, n);
        (o,n)
    }

    /* Store increments in powers of two.  Mipmap is stored as an
    array of arrays.  Unused space is filled with 'neither'. */

    /* FIXME: Wanted to abstract this with slices but can't borrow the
    same array twice? */

    /* Precond: level sizes are correct. */
    #[inline(always)]
    fn build_single<M>(store: &mut [M], level: usize) where M: MipMap {
        let nb_levels = store.len();  // needs to be power of 2.
        // o: offset, n: number of elements
        // c: coarse, f: fine
        let (f_o, _)   = level_o_n(level-1, nb_levels);
        let (c_o, c_n) = level_o_n(level,   nb_levels);
        for c_i in (0..c_n) {
            let f_i = 2 * c_i;
            store[c_o + c_i] =
                MipMap::plane_or(&store[f_o + f_i],
                                 &store[f_o + f_i + 1] );
        }
    }

    /* first level -> compute from raw data. */
    pub fn mipmap<M>(store: &mut [M]) where M: MipMap {
        /* Build first level from store. */
        // TODO: create 2 bit planes

        /* Recursively build other levels. */
        let levels = log2_upper(store.len());
        for level in (1..levels) {
            build_single(store, level);
            // TODO: second bit plane
        }
    }

    /*

    TODO:
    - use disk-backed memory-mapped buffer
    - fix number of channels
    - fix maximum buffer size
    
    -> this makes on-the fly updates really simple + leaves memory
      management to the OS.
    
    */
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
        type Item = u8;
        #[inline(always)]
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
    pub fn stdin8() -> Stdin8 {
        Stdin8 {
            stream: old_io::stdin(),
            buf: [0; 262144],
            offset: 0,
            nb: 0,
        }
    }
    
    #[inline]
    pub fn write_byte(b: u8) {
        let mut out = old_io::stdout();
        let bs = [b];
        match out.write_all(&bs) {
            Err(err) => panic!("{}",err),
            Ok(_) => (),
        }
        match out.flush() {
            Err(err) => panic!("{}",err),
            Ok(_) => (),
        }
    }
}


