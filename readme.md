# infinite shuffle

A library for combinatorics.

A big feature is the ability to specify a big combinatorial space and then iterate over it in random order, without ever hitting the same state twice, and *without* having to generate and hold the entire state-space in memory. We acheive it by, essentially, conceiving the state space as a kind of number system, then iterating the indexes of every element in a random order using a ~~symmetric cipher~~ ~~PCG~~ linear feedback shift register.

```rust
use mako_infinite_shuffle::{Shuffled, Indexing, rng::DefaultShuffler, Cross};
fn main(){
    let d = Shuffled::<_,DefaultShuffler>::new(Cross(0..3, 0..2));
    for i in 0..d.len() {
        println!("{:?}", d.get(i));
    }
}
```
```
(2, 1)
(1, 0)
(2, 0)
(0, 1)
(0, 0)
(1, 1)
```

Basic functionality works, but the library is unfinished for many reasons:

- Immediately after I'd coded it I realized how impractical it was, on possibly every dimension.

    - If the state-space is so large, why do you need to take special steps to guarantee non-collision? (We already assume noncollision for 128 bit hash IDs)
    
    - If you really need an absolute guarantee (rather than a probabilistic guarantee) for whatever metaphysical reasons you have, why not just keep a record of the items you've generated so far and skip the ones that have come up before?
    
    - This wouldn't work for continuous spaces and so I don't think it can work for weighted sampling, which is basically all of the kinds of procgen or sampling that you'd want to do. We live in an analog world (*or at least we live on a complexity level where we can't make sense of the world without modelling it as analog*)

- I couldn't, in the span of less than two hours, find a symmetric block cipher that will produce very very small ciphertexts (*say, 16 bits or less. Even a byte-sized one is annoyingly rough, because that could easily force us to have to re-run the encryption a hundred times in a single iteration. Not terminal, but very lame*).

- It wasn't going to be a very nice API, because rust lacks variadic generics. Right now everything just works over pairs, which works, but it will produce unweildy typed list structures.

I'm keeping it here to document the concept and just in case it turns out there's some weird situation where it makes sense after all.

I'm not sure where this idea came from, but [this post](https://www.brainonfire.net/blog/2021/05/06/cryptographic-shuffle/) also discusses it. There's supposedly some kind of an implementation.

I checked out crypto_secretbox, fpe, and cloudproof_fpe. All impose lower bounds on the length a bitstring is allowed to have. Only a lower bound of 8 would be acceptable, but the lowerbound of cloudproof_fpe is 20.

For non-cryptographic options (faster, easy to adapt to small sizes) a LCG or PCG would be nice, but I'm currently going with a LFSR.