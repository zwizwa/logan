pub mod doodle {
    // EDIT: this is fixed: closure needed 'move'
    
    // Test signal building blocks.
    // ShiftReg: oversampled shift register.

    /// This is currently not possible
    // fn word_bits() -> Iterator<Item=usize> {
    //     (0..nb_bits).map(|bit| (value >> bit) & 1)
    // }

    /// Also, I don't understand how to type closures in return types
    /// (core::marker::Sized not implemented for Fn) and using
    /// closures like below gives lifetime problems.
    //
    

    /// So I'm resorting to a clumsy dual-counter low-level Iterator
    /// struct.

    #[derive(Copy)]
    pub struct ShiftReg {
        reg: usize,
        count: usize,
        bitcount: usize,
        period: usize
    }
    impl Iterator for ShiftReg {
        type Item = usize;
        fn next(&mut self) -> Option<usize> {
            if self.bitcount == 0 {
                self.count -= 1;
                self.reg >>= 1;
                self.bitcount = self.period;
            }
            self.bitcount -= 1;
            if self.count == 0 {
                None
            }
            else {
                let rv = self.reg & 1;
                // println!("bit {}", rv);
                Some(rv)
            }
        }
    }
    pub fn word_bits(nb_bits: usize, period: usize, value: usize) -> ShiftReg {
        ShiftReg{
            reg: value,
            count: nb_bits,
            bitcount: period,
            period: period,
        }
    }
}
