# infinite shuffle

A library for specifying a big combinatorial space and then iterating over it in random order, without ever hitting the same state twice, and *without* having to generate and hold the entire state-space in memory. That is a special feat, and we acheive it by, essentially, conceiving the state-space as an irregular base number system (*which allows methodically iterating through it, generating each possible state, one after the other*), then shuffling the order of the iteration using a symmetric cipher.

I initially thought it might be useful for procedural generation.

It's unfinished, and does not work, for many reasons:

- Immediately after I'd coded it I realized how impractical it was, on possibly every dimension.

    - If the state-space is so large, why do you need to take special steps to guarantee non-collision? (We already assume noncollision for 128 bit hash IDs)
    
    - If you really need an absolute guarantee (rather than a probabilistic guarantee) for whatever metaphysical reasons you have, why not just keep a record of the items you've generated so far and skip the ones that have come up before? It's not credible that this list would grow too long to store even in distributed storage.
    
    - This wouldn't work for continuous spaces and so I don't think it can work for weighted sampling, which is basically all of the kinds of procgen or sampling that you'd want to do. We live in an analog world (*or at least we live on a complexity level where we can't make sense of the world without modelling it as analog*)

- I couldn't, in the span of less than two hours, find a symmetric block cipher that will produce very very small ciphertexts (*say, 16 bits or less. Even a byte-sized one is annoyingly rough, because that could easily force us to have to re-run the encryption a hundred times in a single iteration. Not terminal, but very lame*). It's currently wired up to something that will only go down to 24 bytes, which straight up wont work, it will effectively loop forever. There's no reason to expect people to write cipher algorithms that produce such short ciphertexts, because they can just be rainbow tabled, and ciphers like to make use of every bit of the words they have and will not happily part with them.

- It wasn't going to be a very nice API, because rust lacks variadic generics. Right now everything just works over pairs, which works, but it will produce unweildy typed list structures.

I'm keeping it here to document the concept and just in case it turns out there's some weird situation where it makes sense after all.