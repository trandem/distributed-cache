use std::{thread, time};
use moka::sync::{Cache, CacheBuilder, ConcurrentCacheExt};

pub fn getP (x : usize) -> Option<usize>{
    if x >0 {
        return Some(x);
    }
    None
}
fn main() {
    let cache = CacheBuilder::new(2)
        .build();
    cache.insert(1,"1".to_string());
    cache.insert(2,"2".to_string());
    cache.insert(3,"3".to_string());


    println!("{:?}",cache.get(&3));
    println!("{:?}",cache.get(&2));
    println!("{:?}",cache.get(&1));
    println!("{:?}",cache.get(&1));
}