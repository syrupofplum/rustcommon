use sqlx::mysql::{MySqlRow, MySqlConnectOptions};
use sqlx::Connection;
use futures::TryStreamExt;
use crate::mysqlaccessor::{MySQLAccessor, MySQLAccessorError, MySQLAccessorErrorType};

macro_rules! check_conn_open {
    ($ins:expr) => {
        {
            if $ins.conn.is_none() {
                return Err(MySQLAccessorError {
                    err_type: MySQLAccessorErrorType::ConnNotOpen
                });
            }
        }
    };
}

pub struct MySQLAccessorAsync<'a> {
    pub(crate) host: &'a str,
    pub(crate) port: u16,
    pub(crate) user: &'a str,
    pub(crate) passwd: &'a str,
    pub(crate) db: &'a str,
    pub(crate) charset: &'a str,

    is_open_connection: bool,
    pub(crate) conn: Option<sqlx::MySqlConnection>
}

impl<'a> MySQLAccessor for MySQLAccessorAsync<'a> {
}

impl<'a> MySQLAccessorAsync<'a> {
    pub fn new() -> Self {
        Self {
            host: "localhost",
            port: 3308,
            user: "root",
            passwd: "",
            db: "",
            charset: "utf8",
            is_open_connection: false,
            conn: None,
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
        self.is_open_connection = true;
        if self.conn.is_none() {
            let connect_options = self.get_connect_option();
            println!("{:?}", connect_options);
            self.conn = match sqlx::MySqlConnection::connect_with(&connect_options).await {
                Ok(conn) => Some(conn),
                Err(e) => {
                    println!("{:?}", e);
                    None
                }
            };
        }
        Ok(())
    }

    pub async fn do_sql(&mut self, sql: &str) -> Result<Option<Vec<sqlx::mysql::MySqlRow>>, MySQLAccessorError> {
        if self.conn.is_none() && !self.is_open_connection {
            self.open_connection().await?;
        }
        check_conn_open!(self);
        let map_fetch_row_err: fn(sqlx::Error) -> MySQLAccessorError = move |e| MySQLAccessorError { err_type: MySQLAccessorErrorType::SqlFetchRowError(e) };
        let mut rows = sqlx::query(sql)
            .fetch(self.conn.as_mut().unwrap());
        let mut rst: Vec<MySqlRow> = Vec::new();
        while let Some(row) = rows.try_next().await.map_err(map_fetch_row_err)? {
            rst.push(row);
        }
        Ok(Some(rst))
    }
}
