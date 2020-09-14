use rustcommon::redisaccessor;

use tokio;

fn get_redis_client_test<'a>() -> redisaccessor::RedisAccessor<'a> {
    redisaccessor::RedisAccessor::new()
        .host("localhost")
        .port(6379)
        .pswd("")
        .db(0)
}

#[tokio::test]
async fn test_redis_async_multi_set() -> Result<(), String> {
    let mut redisaccessor = get_redis_client_test();
    let _ = redisaccessor.async_open_conn().await.unwrap();
    let dataset = vec![("test1".to_string(), "jjj".to_string(), 300), ("test2".to_string(), "kkk".to_string(), 300)];
    let rst = redisaccessor.async_multi_setex(dataset).await;
    match rst {
        Ok(_) => Ok(()),
        Err(_) => Err(String::from("do redis_async_multi_set fail"))
    }
}

#[tokio::test]
async fn test_redis_async_multi_setex_expire() -> Result<(), String> {
    let mut redisaccessor = get_redis_client_test();
    let _ = redisaccessor.async_open_conn().await.unwrap();
    let dataset = vec![("test1_nx".to_string(), "jjj2".to_string()), ("test2_nx".to_string(), "kkk2".to_string())];
    let rst = redisaccessor.async_multi_setnx(dataset).await;
    match rst {
        Ok(_) => Ok(()),
        Err(_) => Err(String::from("do redis_async_multi_set fail"))
    }
}
