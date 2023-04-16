use futures::channel::oneshot;
use std::{thread, time::Duration};
use std::collections::HashMap;
use std::sync::Arc;
use futures::FutureExt;


#[tokio::main]
async fn main() {
    let mut cache = HashMap::new();

    let (sender, receiver) = oneshot::channel::<String>();
    cache.insert(1, receiver.shared());
    thread::spawn(|| {
        println!("THREAD: sleeping zzz...");
        thread::sleep(Duration::from_millis(1000));
        println!("THREAD: i'm awake! sending.");
        sender.send(3.to_string()).unwrap();
    });

    println!("MAIN: doing some useful stuff");

    let x = cache.get(&1).unwrap().clone().await;
    println!("{:?}",x);
    let x = cache.get(&1).unwrap().clone().await;
    println!("{:?}",x);
}