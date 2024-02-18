use mako_infinite_shuffle::{light_shuffle, Cross, Indexing};
fn main(){
    let d = light_shuffle(Cross(0..3, 0..2));
    for i in 0..d.len() {
        println!("{:?}", d.get(i));
    }
}