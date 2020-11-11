#[derive(Debug)]
pub struct MySQLAccessorError {
    pub err_type: MySQLAccessorErrorType
}

#[derive(Debug)]
pub enum MySQLAccessorErrorType {
    ConnNotOpen,
    OpenConnError(sqlx::Error),
    SqlSelectError(sqlx::Error),
    SqlFetchRowError(sqlx::Error),
    SqlFetchColumnError(sqlx::Error),
}

pub trait MySQLAccessor {
}
