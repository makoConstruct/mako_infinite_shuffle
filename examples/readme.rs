use mako_infinite_shuffle::{Shuffled, Indexing, rng::DefaultShuffler, Cross};
fn main(){
    let d = Shuffled::<_,DefaultShuffler>::new(Cross(0..3, 0..2));
    for i in 0..d.len() {
        println!("{:?}", d.get(i));
    }
}