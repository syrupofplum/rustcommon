#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub mod mysqlaccessor;
pub mod mysqlaccessor_async;
pub mod mysqlaccessor_pool_async;
pub mod httpaccessor;
pub mod redisaccessor;
pub mod redisaccessor_async;
pub mod redisaccessor_actix;
