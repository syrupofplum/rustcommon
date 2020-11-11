use actix::prelude::*;
use actix_redis::{Command, RedisActor, Error};
use redis_async::{resp::RespValue, resp_array};
use crate::redisaccessor::{RedisAccessorError, RedisAccessorErrorType, RedisAccessor};

macro_rules! check_conn_open {
    ($ins:expr) => {
        {
            if $ins.addr.is_none() {
                return Err(RedisAccessorError {
                    err_type: RedisAccessorErrorType::ConnNotOpen
                });
            }
        }
    };
}

pub struct RedisAccessorActix<'a> {
    pub(crate) host: &'a str,
    pub(crate) port: u16,
    pub(crate) user: &'a str,
    pub(crate) passwd: &'a str,
    pub(crate) db: i64,

    pub(crate) addr: Option<Addr<RedisActor>>
}

impl<'a> RedisAccessor for RedisAccessorActix<'a> {
}

impl<'a> RedisAccessorActix<'a> {
    pub fn new() -> Self {
        Self {
            host: "localhost",
            port: 6380,
            user: "",
            passwd: "",
            db: 0,

            addr: None
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

    pub fn open_connection(&mut self) -> Result<(), RedisAccessorError> {
        if self.addr.is_none() {
            let addr_str = format!("{}:{}", self.host, self.port);
            self.addr = Some(actix_redis::RedisActor::start(addr_str));
        }
        Ok(())
    }

    async fn send_auth(&self) -> Result<bool, RedisAccessorError> {
        check_conn_open!(self);
        let gen_conn_err_closure = || RedisAccessorError {
            err_type: RedisAccessorErrorType::OpenConnError
        };
        let auth_future = self.addr.as_ref().unwrap().send(Command(resp_array!["AUTH", self.passwd]));
        let auth_resp = auth_future.await.map_err(|_| gen_conn_err_closure())?.map_err(|_| gen_conn_err_closure())?;
        match auth_resp {
            RespValue::SimpleString(x) => match x.as_str() {
                "OK" => Ok(true),
                _ => Ok(false)
            },
            _ => Ok(false)
        }
    }

    async fn send_select_db(&self) -> Result<bool, RedisAccessorError> {
        check_conn_open!(self);
        let gen_conn_err_closure = || RedisAccessorError {
            err_type: RedisAccessorErrorType::OpenConnError
        };
        let auth_future = self.addr.as_ref().unwrap().send(Command(resp_array!["SELECT", self.db.to_string()]));
        let auth_resp = auth_future.await.map_err(|_| gen_conn_err_closure())?.map_err(|_| gen_conn_err_closure())?;
        match auth_resp {
            RespValue::SimpleString(x) => match x.as_str() {
                "OK" => Ok(true),
                _ => Ok(false)
            },
            _ => Ok(false)
        }
    }

    async fn match_resp_value(
        &self,
        resp_value: Result<&actix_redis::RespValue, &actix_redis::Error>,
        is_last_cmd_auth: bool
    ) -> Result<String, RedisAccessorErrorType> {
        match resp_value {
            Ok(RespValue::SimpleString(x)) => {
                Ok(format!("{}", x))
            },
            Ok(RespValue::Error(x)) => {
                if x.starts_with("NOAUTH") && !is_last_cmd_auth {
                    self.send_auth().await.map_err(|e| e.err_type)?;
                    self.send_select_db().await.map_err(|e| e.err_type)?;
                    Err(RedisAccessorErrorType::AuthError)
                } else {
                    Ok(format!("{}", x))
                }
            },
            Ok(RespValue::Nil) => {
                Err(RedisAccessorErrorType::GetKeyNotExist)
            },
            Ok(RespValue::BulkString(x)) => {
                Ok(format!("{}", String::from_utf8_lossy(&x).to_string()))
            },
            Ok(RespValue::Integer(x)) => {
                Ok(format!("{}", x))
            },
            Ok(RespValue::Array(x)) => {
                Ok("".to_string())
            },
            Err(e) => {
                Ok(format!("{:?}", e))
            }
        }
    }

    pub async fn get(&self, key: &str) -> Result<String, RedisAccessorError> {
        check_conn_open!(self);
        let gen_err_closure = || RedisAccessorError {
            err_type: RedisAccessorErrorType::GetContentError
        };
        let gen_not_exist_closure = || RedisAccessorError {
            err_type: RedisAccessorErrorType::GetKeyNotExist
        };
        let redis_send_result =  self.addr.as_ref().unwrap().send(Command(resp_array!["GET", key])).await;
        match redis_send_result {
            Ok(resp_value) => match self.match_resp_value(resp_value.as_ref(), false).await {
                Ok(s) => Ok(s),
                Err(e) => match e {
                    RedisAccessorErrorType::AuthError => {
                        let redis_send_result =  self.addr.as_ref().unwrap().send(Command(resp_array!["GET", key])).await;
                        match redis_send_result {
                            Ok(resp_value) => match self.match_resp_value(resp_value.as_ref(), true).await {
                                Ok(s) => Ok(s),
                                Err(e) => Err(gen_not_exist_closure())
                            }
                            _ => Err(gen_err_closure())
                        }
                    },
                    _ => Err(gen_err_closure())
                }
            }
            _ => Err(gen_err_closure())
        }
    }
}
