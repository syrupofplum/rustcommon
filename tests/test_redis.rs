mod test_env;

use rustcommon::redisaccessor;

use tokio;
use log::kv::Source;

fn get_redis_client_test<'a>() -> redisaccessor::RedisAccessor<'a> {
    let get_default = || redisaccessor::RedisAccessor::new()
        .host("localhost")
        .port(6379)
        .passwd("")
        .db(0);
    let env_map = &test_env::ENV_CONFIG;
    if env_map.contains_key("redis.host") &&
        env_map.contains_key("redis.port") &&
        env_map.contains_key("redis.passwd") &&
        env_map.contains_key("redis.db") {
        return redisaccessor::RedisAccessor::new()
            .host(env_map.get("redis.host").unwrap())
            .port(env_map.get("redis.port").unwrap().parse::<u16>().unwrap())
            .passwd(env_map.get("redis.passwd").unwrap())
            .db(env_map.get("redis.db").unwrap().parse::<i64>().unwrap());
    }
    get_default()
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
    let dataset = vec![("test1_nx".to_string(), "jjj2".to_string(), 300), ("test2_nx".to_string(), "kkk2".to_string(), 360)];
    let rst = redisaccessor.async_multi_setnx_expire(dataset).await;
    match rst {
        Ok(_) => Ok(()),
        Err(_) => Err(String::from("do redis_async_multi_set fail"))
    }
}
