use std::io::{self, Read, Write};

/* Manually buffered standard input.  Buffer size such that write from
Saleae driver doesn't need to be chunked. */
pub struct Buf8 {
    buf: [u8; 262144],
    offset: usize
}
impl Iterator for Buf8 {
    type Item = u8;
    #[inline(always)]
    fn next(&mut self) -> Option<u8> {
        loop {
            if self.offset < self.buf.len() {
                let rv = self.buf[self.offset];
                self.offset += 1;
                return Some(rv);
            }
            match io::stdin().read_exact(&mut self.buf) {
                Err(err) => panic!("{}",err),
                Ok(()) => ()
            }
            self.offset = 0;
        }
    }
}
pub fn stdin8() -> Buf8 {
    Buf8 {
        buf: [0; 262144], 
        offset: 262144
    }
}

#[inline]
pub fn write_byte(b: u8) {
    let bs = [b];
    match io::stdout().write(&bs) {
        Err(err) => panic!("{}",err),
        Ok(_) => (),
    }
    match io::stdout().flush() {
        Err(err) => panic!("{}",err),
        Ok(_) => ()
    }
}


