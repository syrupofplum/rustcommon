use crate::redisaccessor::{RedisAccessorError, RedisAccessorErrorType, RedisAccessor};
use redis::{Commands, RedisResult};
use std::collections::HashMap;

macro_rules! check_conn_open {
    ($ins:expr) => {
        {
            if $ins.async_conn.is_none() {
                return Err(RedisAccessorError {
                    err_type: RedisAccessorErrorType::ConnNotOpen
                });
            }
        }
    };
}

pub struct RedisAccessorAsync<'a> {
    pub(crate) host: &'a str,
    pub(crate) port: u16,
    pub(crate) user: &'a str,
    pub(crate) passwd: &'a str,
    pub(crate) db: i64,

    client: Option<redis::Client>,
    pub(crate) async_conn: Option<redis::aio::Connection>
}

impl<'a> RedisAccessor for RedisAccessorAsync<'a> {
}

impl<'a> RedisAccessorAsync<'a> {
    pub fn new() -> Self {
        Self {
            host: "localhost",
            port: 6380,
            user: "",
            passwd: "",
            db: 0,

            client: None,
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

    pub fn passwd(mut self, passwd: &'a str) -> Self {
        self.passwd = passwd;
        self
    }

    pub fn db(mut self, db: i64) -> Self {
        self.db = db;
        self
    }

    pub async fn open_connection(&mut self) -> Result<(), RedisAccessorError> {
        if self.async_conn.is_none() {
            let map_redis_err = |e| RedisAccessorError { err_type: RedisAccessorErrorType::OpenConnError };
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
            if self.passwd != "" {
                connection_info.passwd = Some(String::from(self.passwd));
            }
            let client = redis::Client::open(connection_info).map_err(map_redis_err)?;
            self.client = Some(client);
            self.async_conn = match self.client.as_ref().unwrap().get_async_connection().await {
                Ok(c) => Some(c),
                Err(_e) => None
            };
        }
        Ok(())
    }

    async fn pipe_execute(pipe: &mut redis::Pipeline, async_conn: &mut redis::aio::Connection) -> Result<(), RedisAccessorError> {
        let redis_rst: RedisResult<()> = pipe.query_async(async_conn).await;
        match redis_rst {
            Ok(_) => Ok(()),
            Err(e) => Err(RedisAccessorError {
                err_type: RedisAccessorErrorType::SetContentError
            })
        }
    }

    pub async fn get<T>(&mut self, key: &str) -> Result<T, RedisAccessorError>
        where T: redis::FromRedisValue {
        check_conn_open!(self);
        let rst: RedisResult<T> = redis::cmd("GET").arg(key).query_async(self.async_conn.as_mut().unwrap()).await;
        match rst {
            Ok(r) => Ok(r),
            Err(e) => Err(RedisAccessorError {
                err_type: RedisAccessorErrorType::GetContentError
            })
        }
    }

    pub async fn setex(&mut self, key: &str, val: &str, ex: usize) -> Result<(), RedisAccessorError> {
        check_conn_open!(self);
        let redis_rst: RedisResult<redis::Value> = redis::cmd("SETEX").arg(key).arg(ex).arg(val).query_async(self.async_conn.as_mut().unwrap()).await;
        match redis_rst {
            Ok(_) => Ok(()),
            Err(e) => Err(RedisAccessorError {
                err_type: RedisAccessorErrorType::SetContentError
            })
        }
    }

    pub async fn multi_hmset_expire(&mut self, dataset: Vec<(String, &HashMap<String, String>, usize)>) -> Result<(), RedisAccessorError> {
        check_conn_open!(self);
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
        Self::pipe_execute(pipe, self.async_conn.as_mut().unwrap()).await
    }

    pub async fn multi_setex(&mut self, dataset: Vec<(String, String, usize)>) -> Result<(), RedisAccessorError> {
        check_conn_open!(self);
        if dataset.is_empty() {
            return Ok(());
        }
        let mut pipe = &mut redis::pipe();
        for data in dataset {
            pipe = pipe.cmd("SETEX").arg(data.0).arg(data.2).arg(data.1);
        }
        Self::pipe_execute(pipe, self.async_conn.as_mut().unwrap()).await
    }

    pub async fn multi_setnx(&mut self, dataset: Vec<(String, String)>) -> Result<(), RedisAccessorError> {
        check_conn_open!(self);
        if dataset.is_empty() {
            return Ok(());
        }
        let mut pipe = &mut redis::pipe();
        for data in dataset {
            pipe = pipe.cmd("SETNX").arg(data.0).arg(data.1);
        }
        Self::pipe_execute(pipe, self.async_conn.as_mut().unwrap()).await
    }

    pub async fn multi_setnx_expire(&mut self, dataset: Vec<(String, String, usize)>) -> Result<(), RedisAccessorError> {
        check_conn_open!(self);
        if dataset.is_empty() {
            return Ok(());
        }
        let mut pipe = &mut redis::pipe();
        for data in dataset {
            pipe = pipe.cmd("SETNX").arg(data.0.clone()).arg(data.1);
            pipe = pipe.cmd("EXPIRE").arg(data.0).arg(data.2)
        }
        Self::pipe_execute(pipe, self.async_conn.as_mut().unwrap()).await
    }
}
