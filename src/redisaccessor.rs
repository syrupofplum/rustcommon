use redis::{Commands, RedisResult};
use std::collections::HashMap;

#[derive(Debug)]
pub struct RedisAccessorError {
    err_type: RedisAccessorErrorType
}

#[derive(Debug)]
enum RedisAccessorErrorType {
    OpenConnError(redis::RedisError)
}

pub struct RedisAccessor<'a> {
    pub(crate) host: &'a str,
    pub(crate) port: u16,
    pub(crate) user: &'a str,
    pub(crate) pswd: &'a str,
    pub(crate) db: i64,

    client: Option<redis::Client>,
    pub(crate) conn: Option<redis::Connection>,
    pub(crate) async_conn: Option<redis::aio::Connection>
}

impl<'a> RedisAccessor<'a> {
    pub fn new() -> Self {
        Self {
            host: "localhost",
            port: 6380,
            user: "",
            pswd: "",
            db: 0,

            client: None,
            conn: None,
            async_conn: None
        }
    }

    pub fn host(mut self, host: &'a str) -> Self {
        self.host = host;
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn user(mut self, user: &'a str) -> Self {
        self.user = user;
        self
    }

    pub fn pswd(mut self, pswd: &'a str) -> Self {
        self.pswd = pswd;
        self
    }

    pub fn db(mut self, db: i64) -> Self {
        self.db = db;
        self
    }

    pub fn open_conn(&mut self) -> Result<(), RedisAccessorError> {
        let map_redis_err = |e| RedisAccessorError { err_type: RedisAccessorErrorType::OpenConnError(e) };
        // let connect_uri = format!("redis://{}:{}", self.host, self.port)?;
        let mut connection_info = redis::ConnectionInfo {
            addr: Box::new(redis::ConnectionAddr::Tcp(String::from(self.host), self.port)),
            db: self.db,
            username: None,
            passwd: None
        };
        if self.user != "" {
            connection_info.username = Some(String::from(self.user));
        }
        if self.pswd != "" {
            connection_info.passwd = Some(String::from(self.pswd));
        }
        let client = redis::Client::open(connection_info).map_err(map_redis_err)?;
        self.client = Some(client);
        self.conn = match self.client.as_ref().unwrap().get_connection() {
            Ok(c) => Some(c),
            Err(e) => None
        };
        Ok(())
    }

    pub async fn async_open_conn(&mut self) -> Result<(), RedisAccessorError> {
        let map_redis_err = |e| RedisAccessorError { err_type: RedisAccessorErrorType::OpenConnError(e) };
        // let connect_uri = format!("redis://{}:{}", self.host, self.port)?;
        let mut connection_info = redis::ConnectionInfo {
            addr: Box::new(redis::ConnectionAddr::Tcp(String::from(self.host), self.port)),
            db: self.db,
            username: None,
            passwd: None
        };
        if self.user != "" {
            connection_info.username = Some(String::from(self.user));
        }
        if self.pswd != "" {
            connection_info.passwd = Some(String::from(self.pswd));
        }
        let client = redis::Client::open(connection_info).map_err(map_redis_err)?;
        self.client = Some(client);
        self.async_conn = match self.client.as_ref().unwrap().get_async_connection().await {
            Ok(c) => Some(c),
            Err(e) => None
        };
        Ok(())
    }

    pub fn get(&mut self, key: &str) -> Result<redis::Value, redis::ErrorKind> {
        if self.conn.is_none() {
            return Err(redis::ErrorKind::ClientError);
        }
        let rst: RedisResult<redis::Value> = self.conn.as_mut().unwrap().get(key);
        match rst {
            Ok(r) => Ok(r),
            Err(e) => Err(redis::ErrorKind::ResponseError)
        }
    }

    pub async fn async_get<T: redis::FromRedisValue>(&mut self, key: &str) -> Result<T, redis::ErrorKind> {
        if self.async_conn.is_none() {
            return Err(redis::ErrorKind::ClientError);
        }
        let rst: RedisResult<T> = redis::cmd("GET").arg(key).query_async(self.async_conn.as_mut().unwrap()).await;
        match rst {
            Ok(r) => Ok(r),
            Err(e) => Err(redis::ErrorKind::ResponseError)
        }
    }

    pub fn set(&mut self, key: &str, val: &str, ex: usize) -> Result<(), redis::ErrorKind> {
        if self.conn.is_none() {
            return Err(redis::ErrorKind::ClientError);
        }
        let rst: RedisResult<redis::Value> = self.conn.as_mut().unwrap().set_ex(key, val, ex);
        if rst.is_err() {
            return Err(redis::ErrorKind::ResponseError)
        }
        Ok(())
    }

    pub async fn async_set(&mut self, key: &str, val: &str, ex: usize) -> Result<(), redis::ErrorKind> {
        if self.async_conn.is_none() {
            return Err(redis::ErrorKind::ClientError);
        }
        let rst: RedisResult<redis::Value> = redis::cmd("SETEX").arg(key).arg(ex).arg(val).query_async(self.async_conn.as_mut().unwrap()).await;
        if rst.is_err() {
            return Err(redis::ErrorKind::ResponseError)
        }
        Ok(())
    }

    pub async fn async_multi_hmset(&mut self, dataset: Vec<(String, &HashMap<String, String>, usize)>) -> Result<(), redis::ErrorKind> {
        if self.async_conn.is_none() {
            return Err(redis::ErrorKind::ClientError);
        }
        if dataset.is_empty() {
            return Ok(());
        }
        let mut pipe = &mut redis::pipe();
        for data in dataset {
            let mut kv = Vec::new();
            for (key, val) in data.1 {
                kv.push((key, val));
            }
            pipe = pipe.cmd("HMSET").arg(data.0.clone()).arg(kv.as_slice());
            pipe = pipe.cmd("EXPIRE").arg(data.0).arg(data.2);
        }
        let rst: RedisResult<()> = pipe.query_async(self.async_conn.as_mut().unwrap()).await;
        if rst.is_err() {
            return Err(redis::ErrorKind::ResponseError)
        }
        Ok(())
    }

    pub async fn async_multi_set(&mut self, dataset: Vec<(String, String, usize)>) -> Result<(), redis::ErrorKind> {
        if self.async_conn.is_none() {
            return Err(redis::ErrorKind::ClientError);
        }
        if dataset.is_empty() {
            return Ok(());
        }
        let mut pipe = &mut redis::pipe();
        for data in dataset {
            pipe = pipe.cmd("SETEX").arg(data.0).arg(data.2).arg(data.1);
        }
        let rst: RedisResult<()> = pipe.query_async(self.async_conn.as_mut().unwrap()).await;
        if rst.is_err() {
            return Err(redis::ErrorKind::ResponseError)
        }
        Ok(())
    }
}