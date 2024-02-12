//we're going to use it with state stored somewhere else in some contexts so we compartment it a bit
#[derive(Clone, Copy)]
pub struct LFSRF {
    pub taps: u32,
    pub size: u32,
}

/// a RNG that uses the Linear Feedback Shift Register generation method, which we use for getting compact randomish permutations over naturals under some power of two (and then non-powers of two by repeadly discarding outputs that are out of range), but you can use it for whatever you want.
pub struct LFSR {
    pub lfsrf: LFSRF,
    pub state: u32,
}
impl LFSRF {
    pub fn next(&self, cur: u32) -> u32 {
        //inspired by https://holzhaus.github.io/vinylla/src/vinylla/lfsr.rs.html#172
        (((cur & self.taps).count_ones() & 1) << (self.size - 1)) | (cur >> 1)
    }
    pub fn for_length(length: usize) -> Self {
        Self {
            taps: TAPS[length - 1],
            size: length as u32,
        }
    }
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

impl LFSR {
    pub fn for_length(length: usize) -> Self {
        Self {
            lfsrf: LFSRF::for_length(length),
            state: 1,
        }
    }
    pub fn next(&mut self) -> u32 {
        let r = self.state;
        self.state = self.lfsrf.next(r);
        r
    }
}

impl Iterator for LFSR {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next())
    }
}
