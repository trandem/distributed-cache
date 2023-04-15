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
    cache.insert(1,1);
    cache.insert(2,2);
    cache.insert(3,3);

    // cache.insert(4,4);
    let ten_millis = time::Duration::from_millis(2000);

    thread::sleep(ten_millis);
    println!("{:?}",cache.get(&3));
    println!("{:?}",cache.get(&2));
    println!("{:?}",cache.get(&1));
}