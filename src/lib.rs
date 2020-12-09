#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

#[cfg(any(feature="mysql_async", feature="mysql_pool_async"))]
pub mod mysqlaccessor;
#[cfg(feature="mysql_async")]
pub mod mysqlaccessor_async;
#[cfg(feature="mysql_pool_async")]
pub mod mysqlaccessor_pool_async;
#[cfg(feature="http_async")]
pub mod httpaccessor;
#[cfg(any(feature="redis_async", feature="redis_actix"))]
pub mod redisaccessor;
#[cfg(feature="redis_async")]
pub mod redisaccessor_async;
#[cfg(feature="redis_actix")]
pub mod redisaccessor_actix;
