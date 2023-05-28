use async_trait::async_trait;
use log::error;
use sqlx::{Error, FromRow, MySql, Pool};

use crate::cache_data::dto::{ERROR_DATA, UserData};

#[async_trait]
pub trait DataRepo {
    async fn find_by_key(&self, key: i32) -> Result<UserData, Error>;
    async fn find_by_mul_key(&self, keys: &Vec<i32>) -> Result<Vec<UserData>,Error>;
}


#[derive(Clone, Debug)]
pub struct MySqlDataRepo {
    pub sql_pool: Pool<MySql>,
}

#[async_trait]
impl DataRepo for MySqlDataRepo {
    async fn find_by_key(&self, key: i32) -> Result<UserData, Error> {
        let sql_pool = self.sql_pool.clone();

        let row = sqlx::query_as::<_, UserData>("SELECT id, name from user_data WHERE id = ?")
            .bind(key)
            .fetch_one(&sql_pool)
            .await;
        row
    }

    async fn find_by_mul_key(&self, keys: &Vec<i32>) -> Result<Vec<UserData>,Error> {
        let sql_pool = self.sql_pool.clone();

        let params = format!("?{}", ", ?".repeat(keys.len() - 1));
        let query_str = format!("SELECT id, name FROM user_data WHERE id IN ( { } )", params);
        let mut query = sqlx::query_as::<_, UserData>(&query_str);
        for i in keys.iter() {
            let i = *i;
            query = query.bind(i);
        }

        let rows = query.fetch_all(&sql_pool).await;
        rows
    }
}
