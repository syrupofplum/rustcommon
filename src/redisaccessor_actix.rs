use actix::prelude::*;
use actix_redis::{Command, RedisActor, Error};
use redis_async::{resp::RespValue, resp_array};

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

#[derive(Debug)]
pub struct RedisAccessorError {
    err_type: RedisAccessorErrorType
}

#[derive(Debug)]
enum RedisAccessorErrorType {
    ConnNotOpen,
    AuthError,
    AuthSuccess,
    AuthFailed,
    OpenConnError,
    GetContentError,
    SetContentError
}

pub struct RedisAccessorActix<'a> {
    pub(crate) host: &'a str,
    pub(crate) port: u16,
    pub(crate) user: &'a str,
    pub(crate) passwd: &'a str,
    pub(crate) db: i64,

    pub(crate) addr: Option<Addr<RedisActor>>
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

    pub fn open_connection(&mut self) -> Result<(), RedisAccessorError> {
        let addr_str = format!("{}:{}", self.host, self.port);
        self.addr = Some(actix_redis::RedisActor::start(addr_str));
        Ok(())
    }

    async fn auth(&self) -> Result<bool, RedisAccessorError> {
        check_conn_open!(self);
        let auth_future = self.addr.as_ref().unwrap().send(Command(resp_array!["AUTH", self.passwd]));
        let auth_resp = auth_future.await;
        match auth_resp {
            Ok(r) => match r {
                Ok(RespValue::SimpleString(x)) => {
                    if x == "OK" {
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                },
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
                    self.auth().await;
                    Err(RedisAccessorErrorType::AuthError)
                } else {
                    Ok(format!("{}", x))
                }
            },
            Ok(RespValue::Nil) => {
                Ok("".to_string())
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
        let gen_err_closure = || Err(RedisAccessorError {
            err_type: RedisAccessorErrorType::GetContentError
        });
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
                                Err(e) => gen_err_closure()
                            }
                            _ => gen_err_closure()
                        }
                    },
                    _ => gen_err_closure()
                }
            }
            _ => gen_err_closure()
        }
    }
}
