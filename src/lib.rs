// use cloudproof_fpe::core::Integer;
// use crypto_secretbox::{Key, KeyInit, XSalsa20Poly1305};

use std::hash::Hash;

pub mod lfsr;
use lfsr::LFSRF;

pub trait Memorable: Hash + Eq {}

pub trait Derangement {
    type Item;
    fn len(&self) -> usize;
    fn get(&self, at: usize) -> Self::Item;
    fn iter<'a>(&'a self) -> DerangementIter<'a, Self> {
        DerangementIter::new(self)
    }
    fn map<'a, F, Y>(&'a self, f: F) -> DerangementMap<'a, Self, F>
    where
        F: Fn(Self::Item) -> Y,
    {
        DerangementMap { v: self, f }
    }
}

pub struct DerangementMap<'a, D: ?Sized, F> {
    v: &'a D,
    f: F,
}
impl<'a, D: ?Sized, F, R> Derangement for DerangementMap<'a, D, F>
where
    D: Derangement,
    F: Fn(D::Item) -> R,
{
    type Item = R;
    fn len(&self) -> usize {
        self.v.len()
    }
    fn get(&self, at: usize) -> Self::Item {
        (self.f)(self.v.get(at))
    }
}

pub struct DerangementIter<'a, D: ?Sized> {
    v: &'a D,
    at: usize,
    len: usize,
}
impl<'a, D: ?Sized> DerangementIter<'a, D> {
    pub fn new(v: &'a D) -> Self
    where
        D: Derangement,
    {
        let len = v.len();
        Self { v, at: 0, len }
    }
}
impl<'a, D> Iterator for DerangementIter<'a, D>
where
    D: Derangement,
{
    type Item = D::Item;
    fn next(&mut self) -> Option<Self::Item> {
        if self.at >= self.len {
            None
        } else {
            let r = Some(self.v.get(self.at));
            self.at += 1;
            r
        }
    }
}

#[derive(Hash, PartialEq, Eq)]
pub struct CompoundDerangement<A, B>(pub A, pub B);
impl<A, B, TA, TB> Derangement for CompoundDerangement<A, B>
where
    A: Derangement<Item = TA>,
    B: Derangement<Item = TB>,
{
    type Item = (TA, TB);
    fn len(&self) -> usize {
        self.0.len() * self.1.len()
    }
    fn get(&self, at: usize) -> (TA, TB) {
        let ato = self.1.len();
        let q = at / ato;
        let r = at % ato;
        (self.0.get(q), self.1.get(r))
    }
}
pub struct AtomicDeranger(pub usize);
impl Derangement for AtomicDeranger {
    type Item = usize;
    fn len(&self) -> usize {
        self.0
    }
    fn get(&self, at: usize) -> usize {
        at
    }
}
pub struct ConjunctiveDeranger<A, B>(A, B);
impl<A, B, TA, TB> Derangement for ConjunctiveDeranger<A, B>
where
    A: Derangement<Item = TA>,
    B: Derangement<Item = TB>,
{
    type Item = Result<TA, TB>;
    fn len(&self) -> usize {
        self.0.len() + self.1.len()
    }
    fn get(&self, at: usize) -> Result<TA, TB> {
        let ot = self.0.len();
        if at < ot {
            Ok(self.0.get(at))
        } else {
            Err(self.1.get(at))
        }
    }
}

pub struct LFSRShuffle<D> {
    v: D,
    r: LFSRF,
}
impl<D> LFSRShuffle<D> {
    pub fn new(v: D) -> LFSRShuffle<D>
    where
        D: Derangement,
    {
        let n = (v.len() + 1).next_power_of_two().ilog2();
        Self {
            v,
            r: LFSRF::for_length(n as usize),
        }
    }
}
impl<D> Derangement for LFSRShuffle<D>
where
    D: Derangement,
{
    type Item = D::Item;
    fn len(&self) -> usize {
        self.v.len()
    }
    fn get(&self, at: usize) -> D::Item {
        let mut n = at;
        let st = self.v.len();
        loop {
            n = self.r.next(n as u32) as usize;
            if n < st {
                break;
            }
        }
        self.v.get(n)
    }
}

// pub struct CloudproofShuffle<D> {
//     v: D,
//     key: [u8; 32],
// }
// impl<D> CloudproofShuffle<D> {
//     pub fn from_seed(v: D, seed: u64) -> Self {
//         use rand::{Rng, SeedableRng};
//         Self {
//             v,
//             key: rand::rngs::StdRng::seed_from_u64(seed).gen(),
//         }
//         // let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
//         // let key: [u8;32] = rng.gen();
//         // let cipher = FF1::<Aes256>::new(&key, 2).unwrap();
//         // let nonce = XSalsa20Poly1305::generate_nonce(&mut OsRng); // supposedly, in normal use of a symmetric cypher, this would be unique per message
//         // Self { v, cipher }
//     }
// }
// impl<T, D> Derangement<T> for CloudproofShuffle<D>
// where
//     D: Derangement<T>,
// {
//     fn total(&self) -> usize {
//         self.v.total()
//     }

//     fn draw(&self, at: usize) -> T {
//         let total = self.total() as u64;
//         let l = total.ilog2() + 1;
//         println!("{total}, {l}");
//         let mut rat = at as u64;
//         //wait, this wont ever break, it's (apparently) sampling from the range 0..2^(8*24), but you need it to be sampling from the range of like, 2^8, or whatever the nearest power of 2 is to self.total().
//         loop {
//             rat = Integer::instantiate(2, l as usize)
//                 .unwrap()
//                 .encrypt(&self.key, &[], rat)
//                 .unwrap();
//             // println!("{}", rat);
//             if rat < total {
//                 break;
//             }
//         }
//         self.v.draw(rat as usize)
//     }
// }

//todo: also lcgshuffle (very fast, better statistical properties than lfsr), symmetric cipher shuffle (slow but cryptographically random), pcrng shuffle (better statistical properties than either of the other fast ones)

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compound_lfsr() {
        let an = 8;
        let bn = 7;
        let d = LFSRShuffle::new(CompoundDerangement(AtomicDeranger(an), AtomicDeranger(bn)));
        let sn = d.len();
        let mut i = 0;
        let mut hs = std::collections::HashSet::new();
        for (a, b) in d.iter() {
            assert!(a < an);
            assert!(b < bn);
            assert!(i < sn);
            if hs.contains(&(a, b)) {
                panic!("{:?} was repeated", (a, b));
            }
            hs.insert((a, b));
            i += 1;
        }
        assert_eq!(i, sn);
    }

    #[test]
    fn map() {
        let v:Vec<String> = CompoundDerangement(AtomicDeranger(2),AtomicDeranger(2)).map(|(a,b)| format!("{a}{b}")).iter().collect();
        assert_eq!(&v, &["00", "01", "10", "11"]);
    }

    #[test]
    fn lfsr() {
        fn test_aperiodicity_for_length(length: usize) -> bool {
            let l = lfsr::LFSR::for_length(length);
            let mut s = std::collections::HashSet::new();
            //see that it's aperiodic at least until 3 steps away from the end
            for (i, e) in l.take((1 << length) - 3).enumerate() {
                if s.contains(&e) {
                    println!(
                        "{} iterator repeated itself on the {}th iteration",
                        length, i
                    );
                    return true;
                }
                s.insert(e);
            }
            false
        }
        if test_aperiodicity_for_length(2)
            || test_aperiodicity_for_length(4)
            || test_aperiodicity_for_length(8)
            || test_aperiodicity_for_length(9)
            || test_aperiodicity_for_length(10)
            || test_aperiodicity_for_length(11)
            || test_aperiodicity_for_length(12)
            || test_aperiodicity_for_length(13)
        {
            panic!("oh no, we don't understand");
        }
    }
}
