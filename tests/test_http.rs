use rustcommon::httpaccessor;

use tokio;

#[tokio::test]
async fn test_http_async_get() -> Result<(), String> {
    let resp_wrapper_result = httpaccessor::HttpAccessor::async_get("http://www.baidu.com", 10).await;
    match resp_wrapper_result {
        Ok(resp) => match resp.status_code() {
            200 => Ok(()),
            _ => Err(String::from("do http_async_get fail"))
        },
        Err(e) => {
            Err(String::from("do http_async_get fail"))
        }
    }
}

#[tokio::test]
async fn test_http_async_multi_get() -> Result<(), String> {
    let resp_wrapper_list_result = httpaccessor::HttpAccessor::async_multi_get(&vec!["http://www.baidu.com", "http://www.taobao.com"], 10).await;
    match resp_wrapper_list_result {
        Ok(resp_result_list) => {
            match resp_result_list.into_iter().all(|resp_result| {
                match resp_result {
                    Ok(resp) => match resp.status_code() {
                        200 => true,
                        _ => false
                    }
                    _ => false
                }
            }) {
                true => Ok(()),
                false => Err(String::from("do http_async_multi_get fail"))
            }
        },
        Err(e) => Err(String::from("do http_async_multi_get fail"))
    }
}
