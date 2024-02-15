#![feature(isqrt)]

use std::{hash::Hash, ops::Range};

pub mod lfsr;
use lfsr::LFSRF;

/// if you like shuffling combinatorial objects, you may also like this combinatorial object library, I sure do
pub use number_encoding;

pub trait Memorable: Hash + Eq {}

pub trait Indexing {
    type Item;
    fn len(&self) -> usize;
    fn get(&self, at: usize) -> Self::Item;
}
// pub trait IndexingRef {
//     fn iter<'a, I>(self) -> IndexingIter<Self, I> where Self:Borrow<I> + Sized, I:Indexing;
//     // fn map<F, D, Y>(self, f: F) -> IndexingMap<D, F>
//     // where
//     //     D: Indexing,
//     //     F: Fn(D::Item) -> Y,
//     //     Self: Sized;
// }
// impl<D,I> IndexingRef for D where D:Borrow<I>, I:Indexing {
//     fn iter<'a>(self) -> IndexingIter<Self, I> where Self:Borrow<I> + Sized, I:Indexing {
//         let len = self.borrow().len();
//         IndexingIter {
//             v: self,
//             at: 0,
//             len,
//             _p: PhantomData(),
//         }
//     }
// }

// impl<I> IndexingExtend for Borrow<I> where I:Indexing {}

#[derive(Clone)]
pub struct IndexingMap<I, F> {
    v: I,
    f: F,
}
impl<'a, I, F, R> Indexing for IndexingMap<&'a I, F>
where
    I: Indexing,
    F: Fn(I::Item) -> R,
{
    type Item = R;
    fn len(&self) -> usize {
        self.v.len()
    }
    fn get(&self, at: usize) -> Self::Item {
        (self.f)(self.v.get(at))
    }
}

#[derive(Clone)]
pub struct IndexingIter<D> {
    v: D,
    at: usize,
    len: usize,
}
impl<'a, I> Iterator for IndexingIter<&'a I>
where
    I: ?Sized + Indexing,
{
    type Item = I::Item;
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

pub trait OpsRef {
    fn iter<'a>(&'a self) -> IndexingIter<&'a Self>;
    fn map<'a, F, R>(&'a self, f: F) -> IndexingMap<&'a Self, F> where Self:Indexing, F:Fn(Self::Item)-> R;
}
impl<I> OpsRef for I
where
    I: Indexing + ?Sized,
{
    fn iter<'a>(&'a self) -> IndexingIter<&'a Self> {
        let len = self.len();
        IndexingIter {
            v: self,
            len,
            at: 0,
        }
    }
    fn map<'a, F, R>(&'a self, f: F) -> IndexingMap<&'a Self, F> where Self:Indexing
    {
        IndexingMap{v:self, f}
    }
}

/// this has to be a function because trait objects can't return self types
pub fn iter<'a, I: Indexing + ?Sized>(v: &'a I) -> IndexingIter<&'a I> {
    let len = v.len();
    IndexingIter { v, at: 0, len }
}

#[derive(Clone)]
pub struct Once<T>(pub T);
impl<T> Indexing for Once<T>
where
    T: Clone,
{
    type Item = T;

    fn len(&self) -> usize {
        1
    }

    fn get(&self, _at: usize) -> Self::Item {
        self.0.clone()
    }
}

/// Yeilds the pairing of each element in A with every element in B
#[derive(Hash, PartialEq, Eq, Clone)]
pub struct Cross<A, B>(pub A, pub B);
impl<A, B, TA, TB> Indexing for Cross<A, B>
where
    A: Indexing<Item = TA>,
    B: Indexing<Item = TB>,
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

impl Indexing for Range<usize> {
    type Item = usize;
    fn len(&self) -> usize {
        std::iter::ExactSizeIterator::len(self)
    }
    fn get(&self, at: usize) -> Self::Item {
        self.start + at
    }
}

/// does all of A, then does B
#[derive(Clone)]
pub struct Series<A, B>(A, B);
impl<A, B, TA, TB> Indexing for Series<A, B>
where
    A: Indexing<Item = TA>,
    B: Indexing<Item = TB>,
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

/// Iterates k-sized subsets of the n-sized input set
#[derive(Clone)]
pub struct KSubsets {
    n: usize,
    k: usize,
}
impl KSubsets {
    pub fn new(n: usize, k: usize) -> Self {
        Self { n, k }
    }
}
impl Indexing for KSubsets {
    type Item = Vec<usize>;
    fn len(&self) -> usize {
        number_encoding::combination(self.n, self.k)
    }
    fn get(&self, at: usize) -> Self::Item {
        number_encoding::combinadics::decode(at, self.k)
    }
}

/// Iterates k-sized multiset (where entries are allowed to repeat) subsets of the n-sized input set
#[derive(Clone)]
pub struct KSubmultisets {
    n: usize,
    k: usize,
}
impl KSubmultisets {
    pub fn new(n: usize, k: usize) -> Self {
        Self { n, k }
    }
}
impl Indexing for KSubmultisets {
    type Item = Vec<usize>;
    fn len(&self) -> usize {
        number_encoding::combination(self.n + self.k - 1, self.k)
    }
    fn get(&self, at: usize) -> Self::Item {
        let mut r = number_encoding::combinadics::decode(at, self.k);
        for (i, v) in r.iter_mut().enumerate() {
            *v -= i
        }
        r
    }
}

/// psuedorandomly permutes the given Indexing
#[derive(Clone)]
pub struct LFSRShuffle<D> {
    v: D,
    r: LFSRF,
}
impl<D> LFSRShuffle<D> {
    pub fn new(v: D) -> LFSRShuffle<D>
    where
        D: Indexing,
    {
        let n = (v.len() + 1).next_power_of_two().ilog2();
        Self {
            v,
            r: LFSRF::for_length(n as usize),
        }
    }
}
impl<D> Indexing for LFSRShuffle<D>
where
    D: Indexing,
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

//todo: also lcgshuffle (very fast, better statistical properties than lfsr), symmetric cipher shuffle (slow but cryptographically random), pcrng shuffle (better statistical properties than either of the other fast ones)

// deprecating, use an equivalent number_encoding function instead if you need this. I think this is like choose(n, 2) or something.
// fn triangle_1th(n: usize) -> (usize, usize) {
//     //digit, remainder
//     // numeral = n*(n+1)/2 →
//     // n^2/2 + n/2 - numeral = 0 →
//     // (quadratic formula) n = (-1/2 +- sqrt((1/2)^2 + 4*(1/2)*numeral))/(2*1/2) →
//     // (quadratic formula) n = -1/2 +- sqrt(1/4 + 2*numeral) →
//     // n = -1/2 +- sqrt((1 + 8*numeral)/4) →
//     // n = -1/2 +- sqrt(1 + 8*numeral)/2 →
//     // n = (-1 +- sqrt(1 + 8*numeral))/2 →
//     // (it's not negative) n = (sqrt(1 + 8*numeral) - 1)/2 →
//     // n =
//     let primary = ((1 + 8 * n).isqrt() - 1) / 2;
//     (primary, n - primary * (primary + 1) / 2)
//     // I'm pretty sure the above proof is woo because we're dealing with integers but it still fucking worked out perfectly????
//     //todo: remainder
// }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compound_lfsr() {
        let an = 8;
        let bn = 7;
        let d = LFSRShuffle::new(Cross(0..an, 0..bn));
        let sn = d.len();
        let mut i = 0;
        let mut hs = std::collections::HashSet::new();
        for (a, b) in iter(&d) {
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
        let v: Vec<String> = Cross(0..2, 0..2)
            .map(|(a, b)| format!("{a}{b}"))
            .iter()
            .collect();
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
    // deprecating
    // #[test]
    // fn triangle_second_numeral() {
    //     const expected: &'static [(usize, usize)] = &[
    //         (0, 0),
    //         (1, 0),
    //         (1, 1),
    //         (2, 0),
    //         (2, 1),
    //         (2, 2),
    //         (3, 0),
    //         (3, 1),
    //         (3, 2),
    //         (3, 3),
    //         (4, 0),
    //         (4, 1),
    //         (4, 2),
    //         (4, 3),
    //         (4, 4),
    //         (5, 0),
    //         (5, 1),
    //         (5, 2),
    //         (5, 3),
    //         (5, 4),
    //         (5, 5),
    //     ];
    //     for i in 1..expected.len() {
    //         assert_eq!(triangle_1th(i), expected[i]);
    //     }
    // }

    #[test]
    fn ksubsetsmulti() {
        let k = KSubmultisets::new(2, 3);
        assert_eq!(k.len(), 4);
        let ac: std::collections::HashSet<_> = k.iter().collect();
        assert_eq!(ac.len(), 4);
    }

    #[test]
    fn ksubsetsmulti_format() {
        let k = KSubmultisets::new(2, 3);
        use std::collections::HashSet;
        let mut ac = HashSet::new();
        for e in k.iter() {
            if ac.contains(&e) {
                panic!("duplicate {:?}", &e);
            }
            ac.insert(e);
        }
        let mut cc = HashSet::new();
        cc.insert(vec![0, 1, 1]);
        cc.insert(vec![0, 0, 1]);
        cc.insert(vec![0, 0, 0]);
        cc.insert(vec![1, 1, 1]);
        assert_eq!(&ac, &cc);
    }
    #[test]
    fn ksubsetsmulti_format_more() {
        let k = KSubmultisets::new(4, 2);
        use std::collections::HashSet;
        let mut ac = HashSet::new();
        for e in k.iter() {
            if ac.contains(&e) {
                panic!("duplicate {:?}", &e);
            }
            ac.insert(e);
        }
        let mut cc = HashSet::new();
        cc.insert(vec![0, 0]);
        cc.insert(vec![0, 1]);
        cc.insert(vec![1, 1]);
        cc.insert(vec![0, 2]);
        cc.insert(vec![1, 2]);
        cc.insert(vec![2, 2]);
        cc.insert(vec![0, 3]);
        cc.insert(vec![1, 3]);
        cc.insert(vec![2, 3]);
        cc.insert(vec![3, 3]);
        assert_eq!(&ac, &cc);
    }

    #[test]
    fn ksubsets_format() {
        let k = KSubsets::new(3, 2);
        use std::collections::HashSet;
        let mut ac = HashSet::new();
        for e in k.iter() {
            if ac.contains(&e) {
                panic!("duplicate {:?}", &e);
            }
            ac.insert(e);
        }
        let mut cc = HashSet::new();
        cc.insert(vec![0, 1]);
        cc.insert(vec![0, 2]);
        cc.insert(vec![1, 2]);
        assert_eq!(&ac, &cc);
    }

    #[test]
    fn ksubsets() {
        let k = KSubsets::new(4, 2);
        assert_eq!(k.len(), 6);
        let ac: std::collections::HashSet<_> = iter(&k).collect();
        assert_eq!(ac.len(), 6);
    }

    #[test]
    fn object_safety() {
        let o: Box<dyn Indexing<Item = usize>> = Box::new(0..3);
        o.get(0);
        for _e in o.iter() {
        }
    }
}
