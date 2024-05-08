#![feature(isqrt)]

use std::{borrow::Borrow, hash::Hash, marker::PhantomData, ops::Range};

pub mod rng;
use rng::{Shuffler, DefaultShuffler};

/// if you like shuffling combinatorial objects, you may also like this combinatorial object library, I sure do
pub use number_encoding;

pub trait Memorable: Hash + Eq {}

pub trait Indexing {
    type Item;
    fn len(&self) -> usize;
    fn get(&self, at: usize) -> Self::Item;
    fn into_iter(self) -> IndexingIter<Self, Self>
    where
        Self: Sized,
    {
        let len = self.len();
        IndexingIter {
            v: self,
            len,
            at: 0,
            _i: PhantomData,
        }
    }
    fn into_map<F, R>(self, f: F) -> IndexingMap<Self, Self, F>
    where
        Self: Sized,
        F: Fn(Self::Item) -> R,
    {
        IndexingMap {
            v: self,
            f,
            _i: PhantomData,
        }
    }
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
pub struct IndexingMap<DI, I: ?Sized, F> {
    v: DI,
    f: F,
    _i: PhantomData<I>,
}
impl<DI, I, F, R> Indexing for IndexingMap<DI, I, F>
where
    DI: Borrow<I>,
    I: Indexing + ?Sized,
    F: Fn(I::Item) -> R,
{
    type Item = R;
    fn len(&self) -> usize {
        self.v.borrow().len()
    }
    fn get(&self, at: usize) -> Self::Item {
        (self.f)(self.v.borrow().get(at))
    }
}

#[derive(Clone)]
pub struct IndexingIter<D, I: ?Sized> {
    pub v: D,
    pub at: usize,
    pub len: usize,
    pub _i: PhantomData<I>,
}
impl<'a, DI, I> Iterator for IndexingIter<DI, I>
where
    DI: Borrow<I>,
    I: ?Sized + Indexing,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        if self.at >= self.len {
            None
        } else {
            let r = Some(self.v.borrow().get(self.at));
            self.at += 1;
            r
        }
    }
}
/// I straight up don't know how to abstract over different kinds of references dynamic or not. It may not be possible. I'll just make everything public so that you can do what you need to.
pub fn dyn_iter<I: Indexing + ?Sized>(v: Box<I>) -> IndexingIter<Box<I>, I> {
    let len = <Box<I> as Borrow<I>>::borrow(&v).len();
    IndexingIter {
        v,
        at: 0,
        len,
        _i: PhantomData,
    }
}

/// separated from the above because these are not object-safe
pub trait OpsRef {
    fn iter<'a>(&'a self) -> IndexingIter<&'a Self, Self>;
    fn map<'a, F, R>(&'a self, f: F) -> IndexingMap<&'a Self, Self, F>
    where
        Self: Indexing,
        F: Fn(Self::Item) -> R;
}
impl<I> OpsRef for I
where
    I: Indexing + ?Sized,
{
    fn iter<'a>(&'a self) -> IndexingIter<&'a Self, Self> {
        let len = self.len();
        IndexingIter {
            v: self,
            len,
            at: 0,
            _i: PhantomData,
        }
    }
    fn map<'a, F, R>(&'a self, f: F) -> IndexingMap<&'a Self, Self, F>
    where
        Self: Indexing,
    {
        IndexingMap {
            v: self,
            f,
            _i: PhantomData,
        }
    }
}

// /// this has to be a function because trait objects can't return self types
// pub fn iter<'a, I: Indexing + ?Sized>(v: &'a I) -> IndexingIter<&'a I> {
//     let len = v.len();
//     IndexingIter { v, at: 0, len }
// }

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

#[derive(Clone)]
pub struct IndexVec<T> (pub Vec<T>);
impl<T> Indexing for IndexVec<T> where T:Clone {
    type Item = T;
    fn len(&self) -> usize {
        self.0.len()
    }
    fn get(&self, at: usize) -> Self::Item {
        self.0[at].clone()
    }
}

pub struct Truncate<I>(pub usize, pub I);
impl<I> Indexing for Truncate<I> where I:Indexing {
    type Item=I::Item;

    fn len(&self) -> usize {
        self.0.min(self.1.len())
    }

    fn get(&self, at: usize) -> Self::Item {
        assert!(at < self.len());
        self.1.get(at)
    }
}

/// psuedorandomly permutes the given Indexing
/// ```rust
/// Shuffled::<_, rng::DefaultShuffler>::new(Cross(0..3, 0..2))
/// ```
#[derive(Clone)]
pub struct Shuffled<D, S> {
    v: D,
    r: S,
}
impl<D, S> Shuffled<D, S> {
    pub fn new(v: D) -> Shuffled<D, S>
    where
        D: Indexing,
        S: Shuffler,
    {
        let length = v.len();
        Self {
            v,
            r: S::for_length(length),
        }
    }
}
impl<D, S> Indexing for Shuffled<D, S>
where
    D: Indexing,
    S: Shuffler
{
    type Item = D::Item;
    fn len(&self) -> usize {
        self.v.len()
    }
    fn get(&self, at: usize) -> D::Item {
        let st = self.v.len() as u64;
        let mut n = self.r.output_to_state(at as u64);
        loop {
            n = self.r.next(n as u64) as u64;
            if self.r.state_to_output(n) < st {
                break;
            }
        }
        self.v.get(self.r.state_to_output(n) as usize)
    }
}

pub fn light_shuffle<D>(d:D)-> Shuffled<D, DefaultShuffler> where D:Indexing {
    Shuffled::<D, DefaultShuffler>::new(d)
}
// pub fn heavy_shuffle()-> Shuffled<D, CipherShuffler>

//todo: also lcgshuffle (very fast, better statistical properties than lfsr), symmetric cipher shuffle (slow but cryptographically random), pcrng shuffle (better statistical properties than either of the other fast ones)

#[cfg(test)]
mod tests {
    
    use super::*;
    use self::rng::Rng;
    use std::{cmp::Eq, fmt::Debug, hash::Hash};
    use rng::{LFSRF, LFSRFNTimes};
    
    #[test]
    fn compound_lfsr() {
        let an = 8;
        let bn = 7;
        let d = Shuffled::<_, LFSRF>::new(Cross(0..an, 0..bn));
        let sn = d.len();
        let mut i = 0;
        let mut hs = std::collections::HashSet::new();
        for (a, b) in d.iter() {
            assert!(a < an);
            assert!(b < bn);
            assert!(i < sn);
            println!("{:?}", (a, b));
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
        let c1 = Cross(0..2, 0..2);
        let c2 = c1.clone();
        let v: Vec<String> = c1.map(|(a, b)| format!("{a}{b}")).iter().collect();
        assert_eq!(&v, &["00", "01", "10", "11"]);
        let _c2m = c2.into_map(|(a, b): (usize, usize)| a + b);
    }

    fn test_aperiodicity_for_length<S: Shuffler>(length: usize) -> bool {
        let l = Rng::<S>::for_length(length);
        let mut s = std::collections::HashSet::new();
        //see that it's aperiodic at least until 3 steps away from the end
        for (i, e) in l.take(length).enumerate() {
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

    #[test]
    fn u8_for_all_shufflers() {
        assert!(!test_aperiodicity_for_length::<LFSRF>(256));
        assert!(!test_aperiodicity_for_length::<LFSRFNTimes>(256));
        // assert!(!test_aperiodicity_for_length::<Wrapmuller>(256));
    }

    #[test]
    fn lfsr() {
        if test_aperiodicity_for_length::<LFSRF>(2)
            || test_aperiodicity_for_length::<LFSRF>(4)
            || test_aperiodicity_for_length::<LFSRF>(8)
            || test_aperiodicity_for_length::<LFSRF>(9)
            || test_aperiodicity_for_length::<LFSRF>(10)
            || test_aperiodicity_for_length::<LFSRF>(11)
            || test_aperiodicity_for_length::<LFSRF>(12)
            || test_aperiodicity_for_length::<LFSRF>(13)
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

    use std::collections::HashSet;
    fn hashset_acc_without_repeat<T: Hash + Eq + Debug>(
        all: impl Iterator<Item = T>,
    ) -> HashSet<T> {
        let mut ac = HashSet::new();
        for e in all {
            if ac.contains(&e) {
                panic!("duplicate {:?}", &e);
            }
            ac.insert(e);
        }
        ac
    }
    #[test]
    fn ksubsets_format() {
        let k = KSubsets::new(3, 2);
        let ac = hashset_acc_without_repeat(k.iter());
        let mut cc = HashSet::new();
        cc.insert(vec![0, 1]);
        cc.insert(vec![0, 2]);
        cc.insert(vec![1, 2]);
        assert_eq!(&ac, &cc);
    }

    #[test]
    fn triples_hit_7() {
        let rs: HashSet<Vec<usize>> = hashset_acc_without_repeat(
            Shuffled::<_, LFSRFNTimes>::new(KSubmultisets::new(8, 3))
                .into_iter()
                .take(40),
        );
        println!("{:?}", &rs);
        assert!(rs.iter().any(|v| v.iter().any(|e| *e == 7)), "no 7s. the shuffler was insufficiently random.");
        assert!(rs.iter().any(|v| *v.iter().next().unwrap() == 7), "no 7s. the shuffler was insufficiently random.");
    }

    #[test]
    fn ksubsets() {
        let k = KSubsets::new(4, 2);
        assert_eq!(k.len(), 6);
        let ac: std::collections::HashSet<_> = k.iter().collect();
        assert_eq!(ac.len(), 6);
    }

    #[test]
    fn object_safety() {
        let o: Box<dyn Indexing<Item = usize>> = Box::new(0..3);
        o.get(0);
        for _e in o.iter() {}
        dyn_iter(o);
    }
}
