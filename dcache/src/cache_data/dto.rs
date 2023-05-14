use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, MySql, Pool};

#[derive(Deserialize, Debug)]
pub struct GetCacheByListKeyRequest {
    pub keys: Vec<i32>,
}

#[derive(Serialize, Debug)]
pub struct KeyValue {
    pub key: i32,
    pub value: Option<UserData>,
}

#[derive(Clone, Debug)]
pub struct MySqlDataRepo {
    pub sql_pool: Pool<MySql>,
}

impl MySqlDataRepo {
    pub async fn find_by_key(&self, key: i32) -> UserData {
        let sql_pool = self.sql_pool.clone();

        let row = sqlx::query_as::<_, UserData>("SELECT id, name from user_data WHERE id = ?")
            .bind(key)
            .fetch_one(&sql_pool)
            .await;

        match row {
            Ok(data) => {
                data
            }
            Err(e) => {
                EMPTY_DATA
            }
        }
    }

    pub async fn find_by_mul_key(&self, keys: &Vec<i32>) -> Vec<UserData> {
        let sql_pool = self.sql_pool.clone();

        let params = format!("?{}", ", ?".repeat(keys.len() - 1));
        let query_str = format!("SELECT id, name FROM user_data WHERE id IN ( { } )", params);
        let mut query = sqlx::query_as::<_, UserData>(&query_str);
        for i in keys.iter() {
            let i = *i;
            query = query.bind(i);
        }

        let rows = query.fetch_all(&sql_pool).await;
        match rows {
            Ok(list) => list,
            _ => vec![]
        }
    }
}


pub const EMPTY_DATA: UserData = UserData {
    id: None,
    name: None,
};

#[derive(Serialize, Debug, Clone, FromRow)]
pub struct UserData {
    pub id: Option<i32>,
    pub name: Option<String>,
}