use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::iter::Map;
use std::sync::Arc;

use futures::channel::oneshot;
use futures::channel::oneshot::Receiver;
use futures::future::Shared;
use futures::FutureExt;
use log::{error, info, log};
use sqlx::{MySql, Pool};
use tokio::sync::RwLock;

use crate::cache_data::dto::{EMPTY_DATA, ERROR_DATA, UserData};
use crate::cache_data::dto;
use crate::cache_data::repo::{DataRepo, MySqlDataRepo};

pub struct GlobalCache {
    num_shard: usize,
    shard_max_capacity: usize,
    datasource_center: Vec<RwLock<HashMap<i32, Shared<Receiver<UserData>>>>>,
    data_repo: MySqlDataRepo,
}

impl GlobalCache
{
    pub fn new(num_shard: usize, shard_max_capacity: usize, data_repo: MySqlDataRepo) -> GlobalCache {
        if num_shard == 0 {
            panic!("num_shard is zero");
        }
        if shard_max_capacity == 0 {
            panic!("shard_max_capacity is zero");
        }
        let mut datasource_center = Vec::with_capacity(num_shard);
        for _ in 0..num_shard {
            datasource_center.push(RwLock::new(HashMap::new()));
        }

        GlobalCache {
            num_shard,
            shard_max_capacity,
            datasource_center,
            data_repo,
        }
    }

    pub async fn find_by_key(&self, k: i32) -> Option<UserData> {
        let shard = self.get_shard(k);
        let datasource = self.datasource_center.get(shard).unwrap();
        let lru_cache = datasource.read().await;

        return match lru_cache.get(&k) {
            Some(value) => {
                Some(value.clone().await.unwrap().clone())
            },
            None => {
                drop(lru_cache);
                let mut lru_cache = datasource.write().await;
                // concurrent two or multithreading can reach here so we recheck key exited
                if lru_cache.contains_key(&k) {
                    let value = lru_cache.get(&k).unwrap().clone();
                    return Some(value.await.unwrap().clone());
                }
                // one thread do loading data
                let value = self.find_value_repo(k).await.shared();
                lru_cache.insert(k, value.clone());

                let user_data = value.clone().await.unwrap().clone();

                if user_data == ERROR_DATA {
                    drop(lru_cache);
                    self.invalid(k).await;
                }

                Some(user_data)
            }
        }
    }

    pub async fn find_by_keys(&self, list_key: &Vec<i32>) -> Vec<Option<UserData>> {
        let mut map_future_data = HashMap::with_capacity(list_key.len());
        let mut not_existed_keys = vec![];

        for k in list_key.iter() {
            let k = *k;
            let shard = self.get_shard(k);
            let datasource = self.datasource_center.get(shard).unwrap();
            let lru_cache = datasource.read().await;
            let value_option = lru_cache.get(&k);

            if value_option.is_some() {
                map_future_data.insert(k, Some(value_option.unwrap().clone()));
            } else {
                not_existed_keys.push(k);
            }
        }
        if !not_existed_keys.is_empty() {
            let data_from_repo = self.find_values_from_repo(&not_existed_keys).await;
            map_future_data.extend(data_from_repo);
        }

        let mut output = Vec::with_capacity(list_key.len());

        for (key,future) in map_future_data {
            let value = future.unwrap().await.unwrap().clone();
            if value == ERROR_DATA {
                self.invalid(key).await;
            }
            output.push(Some(value));
        }
        output
    }

    async fn find_values_from_repo(&self, list_keys: &Vec<i32>) -> HashMap<i32,Option<Shared<Receiver<UserData>>>> {
        let mut output = HashMap::new();
        info!("find internet with keys {:?}",list_keys);
        let mut sender_map_by_key = HashMap::new();
        for key in list_keys.iter() {
            let key = *key;
            let (sender, receiver) = oneshot::channel::<UserData>();
            sender_map_by_key.insert(key, sender);

            let shard = self.get_shard(key);

            let datasource = self.datasource_center.get(shard).unwrap();
            let mut lru_cache = datasource.write().await;
            let share_receiver = receiver.shared();
            lru_cache.insert(key, share_receiver.clone());
            output.insert(key,Some(share_receiver));
        }

        let sql_pool = self.data_repo.clone();

        tokio::spawn(async move {
            let keys: Vec<i32> = sender_map_by_key.keys().cloned().collect();

            let rows = sql_pool.find_by_mul_key(&keys).await;
            let mut is_error = false;
            let mut user_data_map_by_key = HashMap::new();
            match rows {
                Ok(values) => {
                    for user_data in values {
                        let key = user_data.id.unwrap();
                        user_data_map_by_key.insert(key, user_data);
                    }
                }
                Err(error) => {
                    error!("query to repo error {:?}", error);
                    is_error = true;
                }
            }

            for key in keys.iter() {
                let key = *key;
                let sender = sender_map_by_key.remove(&key).unwrap();
                if is_error {
                    sender.send(ERROR_DATA).expect("cannot send error at multi");
                } else {
                    match user_data_map_by_key.remove(&key) {
                        None => sender.send(dto::EMPTY_DATA),
                        Some(d) => sender.send(d)
                    }.expect("cannot send data at multi");
                }
            }
        });
        output
    }

    pub async fn invalid(&self, k: i32) -> Option<Shared<Receiver<UserData>>> {
        info!("invalid data {}",k);
        let shard = self.get_shard(k);
        let datasource = self.datasource_center.get(shard).unwrap();
        let mut lru_cache = datasource.write().await;
        lru_cache.remove(&k)
    }

    async fn find_value_repo(&self, k: i32) -> Receiver<UserData> {
        let (sender, receiver) = oneshot::channel::<UserData>();
        let sql_pool = self.data_repo.clone();
        tokio::spawn(async move {
            let result_data = sql_pool.find_by_key(k).await;

            match result_data {
                Ok(user_data) => {
                    sender.send(user_data).expect("cannot send data to receiver");
                }
                Err(sqlx::Error::RowNotFound) => {
                    sender.send(EMPTY_DATA).expect("cannot send empty to receiver");
                }
                Err(e) => {
                    error!("query fail with data {:?} ",e);
                    sender.send(ERROR_DATA).expect("cannot send error to receiver");
                }
            }
        });
        receiver
    }

    fn get_shard(&self, k: i32) -> usize {
        let mut s = DefaultHasher::new();
        k.hash(&mut s);
        let hash_key = s.finish();
        let shard = hash_key % self.num_shard as u64;
        shard as usize
    }
}