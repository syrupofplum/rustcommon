use sqlx::mysql::{MySqlRow, MySqlConnectOptions, MySqlPoolOptions};
use sqlx::Connection;
use futures::TryStreamExt;
use crate::mysqlaccessor_async::MySQLAccessorAsync;
use crate::mysqlaccessor::{MySQLAccessor, MySQLAccessorError, MySQLAccessorErrorType};
use std::sync::Arc;
use tokio::sync::Mutex;

macro_rules! check_conn_pool_open {
    ($ins:expr) => {
        {
            if $ins.conn_pool.is_none() {
                if $ins.conn_pool.is_none() {
                    return Err(MySQLAccessorError {
                        err_type: MySQLAccessorErrorType::ConnNotOpen
                    });
                }
            }
        }
    };
}

#[derive(Clone)]
pub struct MySQLAccessorPoolAsync<'a> {
    pub(crate) host: &'a str,
    pub(crate) port: u16,
    pub(crate) user: &'a str,
    pub(crate) passwd: &'a str,
    pub(crate) db: &'a str,
    pub(crate) charset: &'a str,

    pub(crate) conn_pool: Option<sqlx::MySqlPool>
}

impl<'a> MySQLAccessor for MySQLAccessorPoolAsync<'a> {
}

impl<'a> MySQLAccessorPoolAsync<'a> {
    pub fn new() -> Self {
        Self {
            host: "localhost",
            port: 3308,
            user: "root",
            passwd: "",
            db: "",
            charset: "utf8",
            conn_pool: None,
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

    pub fn db(mut self, db: &'a str) -> Self {
        self.db = db;
        self
    }

    pub fn charset(mut self, charset: &'a str) -> Self {
        self.charset = charset;
        self
    }

    fn get_connect_option(&self) -> MySqlConnectOptions {
        sqlx::mysql::MySqlConnectOptions::new()
            .username(self.user)
            .password(self.passwd)
            .host(self.host)
            .port(self.port)
            .database(self.db)
            .charset(self.charset)
    }

    pub async fn open_connection(&mut self) -> Result<(), MySQLAccessorError> {
        if self.conn_pool.is_none() {
            // let connect_uri = format!("mysql://{}:{}@{}:{}/{}", self.user, self.passwd, self.host, self.port, self.db);
            let connect_options = self.get_connect_option();
            self.conn_pool = match MySqlPoolOptions::new()
                .max_connections(2)
                .connect_with(connect_options)
                .await
            {
                Ok(pool) => Some(pool.clone()),
                Err(_e) => None
            };
        }
        Ok(())
    }

    pub async fn do_sql(&self, sql: &str) -> Result<Option<Vec<sqlx::mysql::MySqlRow>>, MySQLAccessorError> {
        check_conn_pool_open!(self);
        let map_fetch_row_err: fn(sqlx::Error) -> MySQLAccessorError = move |e| MySQLAccessorError { err_type: MySQLAccessorErrorType::SqlFetchRowError(e) };
        let mut rows = sqlx::query(sql)
            .fetch(self.conn_pool.as_ref().unwrap());
        let mut rst: Vec<MySqlRow> = Vec::new();
        while let Some(row) = rows.try_next().await.map_err(map_fetch_row_err)? {
            rst.push(row);
        }
        Ok(Some(rst))
    }
}
