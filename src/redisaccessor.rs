use std::collections::HashMap;

#[derive(Debug)]
pub struct RedisAccessorError {
    pub err_type: RedisAccessorErrorType
}

#[derive(Debug)]
pub enum RedisAccessorErrorType {
    ConnNotOpen,
    OpenConnError,
    AuthError,
    GetContentError,
    GetKeyNotExist,
    SetContentError
}

pub trait RedisAccessor {
}
