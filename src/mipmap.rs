
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

pub trait MipMap: Sized {
    fn plane_init(&self) -> (Self, Self);      // orig -> level 0
    fn plane_or(&self, other: &Self) -> Self;  // level n -> level n + 1
}
// Is there a trait to abstract logic operations instead of this workaround?
macro_rules! impl_MipMap {
    ($t:ty) => (
        impl MipMap for $t {
            fn plane_init(&self) -> ($t,$t) {
                let bit1 = *self;
                let bit0 = !bit1;
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
    let left_ones  = (!0) << (nb_levels - level + 1);
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
    for c_i in 0..c_n {
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
    for level in 1..levels {
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


