use crypto_secretbox::{Key, KeyInit, XSalsa20Poly1305};

pub trait Derangement<T> {
    fn total(&self) -> usize;
    fn draw(&self, at: usize) -> T;
}
pub struct CompoundDerangement<A, B> {
    a: A,
    b: B,
}
impl<A, B> CompoundDerangement<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}
impl<A, B, TA, TB> Derangement<(TA, TB)> for CompoundDerangement<A, B>
where
    A: Derangement<TA>,
    B: Derangement<TB>,
{
    fn total(&self) -> usize {
        self.a.total() * self.b.total()
    }
    fn draw(&self, at: usize) -> (TA, TB) {
        let ato = self.a.total();
        let q = at / ato;
        let r = at % ato;
        (self.a.draw(q), self.b.draw(r))
    }
}
pub struct AtomicDeranger(pub usize);
impl Derangement<usize> for AtomicDeranger {
    fn total(&self) -> usize {
        self.0
    }
    fn draw(&self, at: usize) -> usize {
        at
    }
}
pub struct ConjunctiveDeranger<A, B>(A, B);
impl<A, B, TA, TB> Derangement<Result<TA, TB>> for ConjunctiveDeranger<A, B>
where
    A: Derangement<TA>,
    B: Derangement<TB>,
{
    fn total(&self) -> usize {
        self.0.total() + self.1.total()
    }
    fn draw(&self, at: usize) -> Result<TA, TB> {
        let ot = self.0.total();
        if at < ot {
            Ok(self.0.draw(at))
        } else {
            Err(self.1.draw(at))
        }
    }
}
pub struct Shuffle<D> {
    v: D,
    cipher: XSalsa20Poly1305,
}
impl<D> Shuffle<D> {
    pub fn from_seed(v: D, seed: u64) -> Self {
        use rand::{RngCore, SeedableRng};
        let mut key = Key::default();
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        rng.fill_bytes(&mut key);
        let cipher = XSalsa20Poly1305::new(&key);
        // let nonce = XSalsa20Poly1305::generate_nonce(&mut OsRng); // supposedly, in normal use of a symmetric cypher, this would be unique per message
        Self { v, cipher }
    }
}
impl<T, D> Derangement<T> for Shuffle<D>
where
    D: Derangement<T>,
{
    fn total(&self) -> usize {
        self.v.total()
    }

    fn draw(&self, at: usize) -> T {
        use crypto_secretbox::{aead::Aead, Nonce};
        use rand::{RngCore, SeedableRng};
        let mut nonce = Nonce::default();
        let mut rng = rand::rngs::StdRng::seed_from_u64(90);
        rng.fill_bytes(&mut nonce);
        let mut rat = at as u64;
        let ot = self.total() as u64;
        //wait, this wont ever break, it's (apparently) sampling from the range 0..2^(8*24), but you need it to be sampling from the range of like, 2^8, or whatever the nearest power of 2 is to self.total().
        loop {
            //this is apparently 24 bytes long, rather than 8, which is a big problem (that's why try_into breaks)
            let encrypted = self
                .cipher
                .encrypt(&nonce, rat.to_be_bytes().as_slice())
                .unwrap();
            assert_eq!(
                encrypted.len(),
                8,
                "if it's longer than the input, then we can't make this efficient"
            );
            rat = u64::from_be_bytes(encrypted.as_slice().try_into().unwrap());
            if rat < ot {
                break;
            }
        }
        self.v.draw(rat as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn main_example() {
        let d = Shuffle::from_seed(
            CompoundDerangement::new(AtomicDeranger(8), AtomicDeranger(7)),
            400,
        );
        for i in 0..30.min(d.total()) {
            println!("{:?}", d.draw(i));
        }
    }
}

