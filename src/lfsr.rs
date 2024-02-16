pub trait Shuffler {
    fn for_length(l: usize) -> Self;
    fn next(&self, prev: u64) -> u64;
    fn state_to_output(&self, state: u64) -> u64 {
        state
    }
    fn output_to_state(&self, state: u64) -> u64 {
        state
    }
    fn initial_state(length: usize) -> u64 {
        0x2ab18f32a337u64 % length as u64
    }
}

// pub struct Lcg { m:u64, c:u64, };
// impl Shuffler for Wrapmuller {
//     fn for_length(l: usize) -> Self {
//         assert!(l == 256);
//         Wrapmuller(8)
//     }
//     fn next(&self, prev: u64) -> u64 {
//         (prev as u8).wrapping_mul(217u8) as u64
//     }
// }

//we're going to use it with state stored somewhere else in some contexts so we compartment it a bit
#[derive(Clone, Copy)]
pub struct LFSRF {
    pub taps: u32,
    pub size: u32,
}
impl Shuffler for LFSRF {
    fn initial_state(length: usize) -> u64 {
        let special_salt = 0x2ab18f32a337u64; //shrug
        let mut state = special_salt % length as u64;
        if state == 0 {
            state = 1;
        }
        state
    }
    fn next(&self, prev: u64) -> u64 {
        //inspired by https://holzhaus.github.io/vinylla/src/vinylla/lfsr.rs.html#172
        (((prev & self.taps as u64).count_ones() as u64 & 1) << (self.size - 1)) | (prev >> 1)
    }
    /// for period l. Should return with a period above and close to l, but doesn't have to be l exactly (the point of full period is that we can just try again if we get one that's too long, and if you're close enough to the correct period you have a probabilistic guarantee that you wont have to try too many times).
    fn for_length(l: usize) -> Self {
        // + 1 because a lfsr actually skips the 0
        let bl = (l + 1).ilog2() + 1;
        Self {
            taps: TAPS[(bl - 1) as usize],
            size: bl as u32,
        }
    }
    fn state_to_output(&self, state: u64) -> u64 {
        //a lfsr never generates 0
        state - 1
    }
    fn output_to_state(&self, state: u64) -> u64 {
        //a lfsr never generates 0
        state + 1
    }
}

/// a RNG that uses the Linear Feedback Shift Register generation method, which we use for getting compact randomish permutations over naturals under some power of two (and then non-powers of two by repeadly discarding outputs that are out of range), but you can use it for whatever you want.
#[derive(Clone, Copy)]
pub struct Rng<Core> {
    pub core: Core,
    pub length: u64,
    pub state: u64,
}

//tap table was translated from https://github.com/ilya-epifanov/lfsr/blob/8fe2078730a10ba42c2e2f4fb7849b79b9407fb8/instances/src/galois.rs#L4 using the commented out code below. That library in turn got them from [Table of Linear Feedback Shift Registers](http://courses.cse.tamu.edu/walker/csce680/lfsr_table.pdf) by Roy Ward, Tim Molteno
const TAPS: [u32; 32] = [
    0x1, 0x3, 0x3, 0x3, 0x5, 0x3, 0x3, 0x1d, 0x11, 0x9, 0x5, 0x53, 0x1b, 0x2b, 0x3, 0x2d, 0x9,
    0x81, 0x27, 0x9, 0x5, 0x3, 0x21, 0x1b, 0x9, 0x47, 0x27, 0x9, 0x5, 0x53, 0x9, 0xc5,
];
// The above translation was generated with the code below
// pub fn tap_table() -> [u32; 32] {
//     //first entry (single bit) is duff. I guess a one bit lfsr wouldn't be able to count at all because LFSRs can't do the zero state.
//     let tap_bit_addresses: [&'static [usize]; 32] = [
//         &[32, 30, 26, 25],
//         &[31, 28],
//         &[30, 29, 26, 24],
//         &[29, 27],
//         &[28, 25],
//         &[27, 26, 25, 22],
//         &[26, 25, 24, 20],
//         &[25, 22],
//         &[24, 23, 21, 20],
//         &[23, 18],
//         &[22, 21],
//         &[21, 19],
//         &[20, 17],
//         &[19, 18, 17, 14],
//         &[18, 11],
//         &[17, 14],
//         &[16, 14, 13, 11],
//         &[15, 14],
//         &[14, 13, 11, 9],
//         &[13, 12, 10, 9],
//         &[12, 11, 8, 6],
//         &[11, 9],
//         &[10, 7],
//         &[9, 5],
//         &[8, 6, 5, 4],
//         &[7, 6],
//         &[6, 5],
//         &[5, 3],
//         &[4, 3],
//         &[3, 2],
//         &[2, 1],
//         &[1],
//     ];
//     tap_bit_addresses
//         .into_iter()
//         .rev()
//         .enumerate()
//         .map(|(i, ar)| {
//             let mut ret: u64 = 0;
//             for ba in ar.iter() {
//                 ret |= 1 << (i + 1 - ba);
//             }
//             ret as u32
//         })
//         .collect::<Vec<u32>>()
//         .try_into()
//         .unwrap()
// }

impl<Core: Shuffler> Rng<Core> {
    pub fn for_length(length: usize) -> Self {
        Self {
            core: Core::for_length(length),
            length: length as u64,
            state: Core::initial_state(length),
        }
    }
    pub fn next(&mut self) -> u64 {
        let r = self.state;
        // shouldn't loop long, as each iteration has an uncorrelated probability of being below range, for most shuffler's it's better odds than a coin flip each time. Shufflers should have a full period (and are tested) so looping forever should be impossible.
        loop {
            self.state = self.core.next(self.state);
            if self.core.state_to_output(self.state) < self.length {
                break;
            }
        }
        self.core.state_to_output(r)
    }
}

impl<Core> Iterator for Rng<Core>
where
    Core: Shuffler,
{
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        Some(Rng::next(self))
    }
}
