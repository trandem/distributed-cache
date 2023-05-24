use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use futures::channel::oneshot;
use futures::channel::oneshot::Receiver;
use futures::future::Shared;
use futures::FutureExt;
use log::{error, info};
use tokio::sync::RwLock;

use crate::cache_data::dto::{ERROR_DATA, MySqlDataRepo, UserData};
use crate::cache_data::dto;

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

    pub async fn find_by_key(&self, k: i32) -> Option<Shared<Receiver<UserData>>> {
        let shard = self.get_shard(k);
        let datasource = self.datasource_center.get(shard).unwrap();
        let lru_cache = datasource.read().await;
        let value_option = lru_cache.get(&k);

        if value_option.is_some() {
            return Some(value_option.unwrap().clone());
        } else {
            drop(lru_cache);
            let mut lru_cache = datasource.write().await;
            // concurrent two or multithreading can reach here so we recheck key exited
            if lru_cache.contains_key(&k) {
                let value_option = lru_cache.get_mut(&k);
                if value_option.is_none() {
                    return None;
                }
                return Some(value_option.unwrap().clone());
            }
            // one thread do loading data
            let value = self.find_value_repo(k).await;
            lru_cache.insert(k, value.shared());

            let value_option = lru_cache.get(&k);
            if value_option.is_none() {
                return None;
            }
            return Some(value_option.unwrap().clone());
        }
    }

    pub async fn find_by_keys(&self, list_key: &Vec<i32>) -> Vec<Option<Shared<Receiver<UserData>>>> {
        let mut output = Vec::with_capacity(list_key.len());
        let mut not_existed_keys = vec![];

        for k in list_key.iter() {
            let k = *k;
            let shard = self.get_shard(k);
            let datasource = self.datasource_center.get(shard).unwrap();
            let lru_cache = datasource.read().await;
            let value_option = lru_cache.get(&k);

            if value_option.is_some() {
                output.push(Some(value_option.unwrap().clone()))
            } else {
                not_existed_keys.push(k);
            }
        }
        if !not_existed_keys.is_empty() {
            self.find_values_from_repo(&not_existed_keys, &mut output).await;
        }
        output
    }

    async fn find_values_from_repo(&self, list_keys: &Vec<i32>, results: &mut Vec<Option<Shared<Receiver<UserData>>>>) {
        info!("find internet with keys {:?}",list_keys);
        let mut data = HashMap::new();
        for key in list_keys.iter() {
            let key = *key;
            let (sender, receiver) = oneshot::channel::<UserData>();
            data.insert(key, sender);

            let shard = self.get_shard(key);

            let datasource = self.datasource_center.get(shard).unwrap();
            let mut lru_cache = datasource.write().await;
            lru_cache.insert(key, receiver.shared());
            let value_option = lru_cache.get(&key);

            results.push(Some(value_option.unwrap().clone()));
        }

        let sql_pool = self.data_repo.clone();

        tokio::spawn(async move {
            let keys: Vec<i32> = data.keys().cloned().collect();

            let rows = sql_pool.find_by_mul_key(&keys).await;

            let mut map = rows.iter()
                .map(|data| (data.id.unwrap(), data.clone()))
                .collect::<HashMap<i32, UserData>>();

            for key in keys.iter() {
                let key = *key;
                let sender = data.remove(&key).unwrap();
                match map.remove(&key) {
                    None => sender.send(dto::EMPTY_DATA),
                    Some(d) => sender.send(d)
                }.expect("TODO: panic message");
            }
        });
    }

    pub async fn invalid(&self, k: i32) -> Option<Shared<Receiver<UserData>>> {
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
            if result_data.is_ok() {
                let user_data = result_data.unwrap();
                sender.send(user_data)
            } else {
                error!("query fail with data {:?} ",result_data.err());
                sender.send(ERROR_DATA)
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