mod test_env;

use rustcommon::mysqlaccessor;

use tokio;
use sqlx::Row;

fn get_mysql_client_test<'a>() -> mysqlaccessor::MySQLAccessor<'a> {
    let env_map_option = test_env::init();
    let get_default = || mysqlaccessor::MySQLAccessor::new()
        .host("localhost")
        .port(3306)
        .user("root")
        .passwd("")
        .db("test")
        .charset("utf8");
    match env_map_option {
        Some(env_map) => {
            if env_map.contains_key("mysql.host") && env_map.contains_key("mysql.port") {
                return get_default();
            }
            get_default()
        },
        None => get_default()
    }

}

#[tokio::test]
async fn test_mysql_async_select() -> Result<(), String> {
    let mut mysql_client = get_mysql_client_test();
    let _ = mysql_client.async_open_conn().await.unwrap();
    match mysql_client.async_do_sql("").await {
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
            None => Err(String::from("do mysql_async_select fail"))
        },
        Err(_) => Err(String::from("do mysql_async_select fail"))
    }
}
