mod test_env;

use rustcommon::mysqlaccessor_async;

use tokio;
use sqlx::Row;

fn get_mysql_client_test<'a>() -> mysqlaccessor_async::MySQLAccessorAsync<'a> {
    let get_default = || mysqlaccessor_async::MySQLAccessorAsync::new()
        .host("localhost")
        .port(3306)
        .user("root")
        .passwd("")
        .db("test")
        .charset("utf8");
    let env_map = &test_env::ENV_CONFIG;
    if env_map.contains_key("mysql.host") &&
        env_map.contains_key("mysql.port") &&
        env_map.contains_key("mysql.user") &&
        env_map.contains_key("mysql.passwd") &&
        env_map.contains_key("mysql.db") {
        return mysqlaccessor_async::MySQLAccessorAsync::new()
            .host(env_map.get("mysql.host").unwrap().as_str())
            .port(env_map.get("mysql.port").unwrap().parse::<u16>().unwrap())
            .user(env_map.get("mysql.user").unwrap().as_str())
            .passwd(env_map.get("mysql.passwd").unwrap().as_str())
            .db(env_map.get("mysql.db").unwrap().as_str())
            .charset("utf8");
    }
    get_default()
}

#[tokio::test]
async fn test_mysql_async_select() -> Result<(), String> {
    let mut mysql_client = get_mysql_client_test();
    let _ = mysql_client.open_connection().await.unwrap();
    match mysql_client.do_sql("select `id` from test_table").await {
        Ok(rows_option) => match rows_option {
            Some(rows) => {
                if rows.len() == 0 {
                    return Err(String::from("do mysql_async_select fail, empty result"));
                }
                match rows.into_iter().all(|row| {
                    true
                }) {
                    true => Ok(()),
                    false => Err(String::from("do mysql_async_select fail"))
                }
            }
            None => {Err(String::from("do mysql_async_select fail"))}
        },
        Err(_) => {Err(String::from("do mysql_async_select fail"))}
    }
}
