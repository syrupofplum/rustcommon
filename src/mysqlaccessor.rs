use futures::TryStreamExt;
use sqlx::mysql::{MySqlRow, MySqlConnectOptions};
use sqlx::Connection;

macro_rules! check_conn_open {
    ($ins:expr) => {
        {
            if $ins.conn.is_none() {
                return Err(MysqlAccessorError {
                    err_type: MysqlAccessorErrorType::ConnNotOpen
                });
            }
        }
    };
}

macro_rules! check_conn_pool_open {
    ($ins:expr) => {
        {
            if $ins.conn_pool.is_none() {
                return Err(MysqlAccessorError {
                    err_type: MysqlAccessorErrorType::ConnNotOpen
                });
            }
        }
    };
}

#[derive(Debug)]
pub struct MysqlAccessorError {
    err_type: MysqlAccessorErrorType
}

#[derive(Debug)]
enum MysqlAccessorErrorType {
    ConnNotOpen,
    OpenConnError(sqlx::Error),
    SqlSelectError(sqlx::Error),
    SqlFetchRowError(sqlx::Error),
    SqlFetchColumnError(sqlx::Error),
}

pub struct MySQLAccessor<'a> {
    pub(crate) host: &'a str,
    pub(crate) port: u16,
    pub(crate) user: &'a str,
    pub(crate) passwd: &'a str,
    pub(crate) db: &'a str,
    pub(crate) charset: &'a str,

    pub(crate) conn: Option<sqlx::MySqlConnection>,
    pub(crate) conn_pool: Option<sqlx::MySqlPool>
}

impl<'a> MySQLAccessor<'a> {
    pub fn new() -> Self {
        Self {
            host: "localhost",
            port: 3308,
            user: "root",
            passwd: "",
            db: "",
            charset: "utf8",
            conn: None,
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

    pub async fn async_open_conn(&mut self) -> Result<(), MysqlAccessorError> {
        let connect_options = self.get_connect_option();
        println!("{:?}", connect_options);
        self.conn = match sqlx::MySqlConnection::connect_with(&connect_options).await {
            Ok(conn) => Some(conn),
            Err(e) => {println!("{:?}", e);None}
        };
        Ok(())
    }

    pub async fn async_open_conn_pool(&mut self) -> Result<(), MysqlAccessorError> {
        // let connect_uri = format!("mysql://{}:{}@{}:{}/{}", self.user, self.passwd, self.host, self.port, self.db);
        let connect_options = self.get_connect_option();
        self.conn_pool = match sqlx::MySqlPool::connect_with(connect_options).await {
            Ok(pool) => Some(pool),
            Err(_e) => None
        };
        Ok(())
    }

    pub async fn async_do_sql(&mut self, sql: &str) -> Result<Option<Vec<sqlx::mysql::MySqlRow>>, MysqlAccessorError> {
        check_conn_open!(self);
        let map_fetch_row_err: fn(sqlx::Error) -> MysqlAccessorError = move |e| MysqlAccessorError { err_type: MysqlAccessorErrorType::SqlFetchRowError(e) };
        let mut rows = sqlx::query(sql)
            .fetch(self.conn.as_mut().unwrap());
        let mut rst: Vec<MySqlRow> = Vec::new();
        while let Some(row) = rows.try_next().await.map_err(map_fetch_row_err)? {
            rst.push(row);
        }
        Ok(Some(rst))
    }

    pub async fn async_do_sql_pool(&self, sql: &str) -> Result<Option<Vec<sqlx::mysql::MySqlRow>>, MysqlAccessorError> {
        check_conn_pool_open!(self);
        let map_fetch_row_err: fn(sqlx::Error) -> MysqlAccessorError = move |e| MysqlAccessorError { err_type: MysqlAccessorErrorType::SqlFetchRowError(e) };
        let mut rows = sqlx::query(sql)
            .fetch(self.conn_pool.as_ref().unwrap());
        let mut rst: Vec<MySqlRow> = Vec::new();
        while let Some(row) = rows.try_next().await.map_err(map_fetch_row_err)? {
            rst.push(row);
        }
        Ok(Some(rst))
    }
}
