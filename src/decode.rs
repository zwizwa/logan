pub struct Decode<'a,I,S,T:'a,O>
    where for<O> S: Iterator<Item=I>, T: Tick<I,O>
{ s: S, t: &'a mut T,  }

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
