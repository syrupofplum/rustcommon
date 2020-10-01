use tokio::time::Duration;
use futures::{future, StreamExt, Future};
use futures::task::{Context, Poll};
use tokio::macros::support::Pin;

#[derive(Debug)]
pub struct HttpAccessorError {
    err_type: HttpAccessorErrorType,
    msg: String
}

#[derive(Debug)]
enum HttpAccessorErrorType {
    OpenUrlError(reqwest::Error),
    GetContentError(reqwest::Error),
    GetMultiContentError(reqwest::Error)
}

pub struct HttpAccessorResponse {
    pub(crate) url: String,
    pub(crate) status_code: u16,
    pub(crate) content: String,
}

impl HttpAccessorResponse {
    pub fn url(&self) -> &str {
        self.url.as_str()
    }
    pub fn status_code(&self) -> u16 {
        self.status_code
    }
    pub fn content(&self) -> &str {
        self.content.as_str()
    }
}

#[derive(Debug)]
pub struct HttpAccessorResponseError {
    pub(crate) url: String,
    pub(crate) status_code: Option<u16>,
    pub(crate) err: Option<HttpAccessorError>
}

impl HttpAccessorResponseError {
    pub fn url(&self) -> &str {
        self.url.as_str()
    }
    pub fn status_code(&self) -> &Option<u16> {
        &self.status_code
    }
    pub fn err(&self) -> &Option<HttpAccessorError> {
        &self.err
    }
}

struct HttpAccessorResponseInner {
    status_code: u16,
    content: String,
}

pub struct HttpAccessor {
    client: Option<reqwest::Client>,
    pub(crate) timeout: u32
}

struct DummyFutureError {
}

impl future::Future for DummyFutureError {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        unimplemented!()
    }
}

impl HttpAccessor {
    pub fn new() -> Self{
        Self {
            client: None,
            timeout: 60
        }
    }

    pub fn timeout(mut self, timeout: u32) -> Self {
        self.timeout = timeout;
        self
    }

    async fn handle_async_resp(url: &str, resp: reqwest::Response) -> Result<HttpAccessorResponse, HttpAccessorResponseError> {
        let status_code = resp.status();
        let server_info = resp.text().await.map_err(|e| HttpAccessorResponseError {
            url: url.to_string(),
            status_code: Some(status_code.as_u16()),
            err: Some(HttpAccessorError {
                err_type: HttpAccessorErrorType::GetContentError(e),
                msg: "async get text fail".to_string()
            })
        })?;
        Ok(HttpAccessorResponse{
            url: url.to_string(),
            status_code: status_code.as_u16(),
            content: server_info
        })
    }

    pub async fn async_get(url: &str, timeout: u32) -> Result<HttpAccessorResponse, HttpAccessorResponseError> {
        let resp = reqwest::Client::new()
            .get(url)
            .timeout(Duration::from_secs(timeout as u64))
            .send()
            .await
            .map_err(|e| HttpAccessorResponseError {
                url: url.to_string(),
                status_code: None,
                err: Some(HttpAccessorError {
                    err_type: HttpAccessorErrorType::OpenUrlError(e),
                    msg: "async get fail".to_string()
                })
            })?;
        HttpAccessor::handle_async_resp(url, resp).await
    }

    pub fn async_post<'a>(url: &'a str, body: &str, timeout: u32) -> impl Future<Output = Result<HttpAccessorResponse, HttpAccessorResponseError>> + 'a {
        let request_builder = reqwest::Client::new()
            .post(url)
            .body(body.to_string())
            .timeout(Duration::from_secs(timeout as u64));
        async move {
            let resp = request_builder
                .send()
                .await
                .map_err(|e| HttpAccessorResponseError {
                    url: url.to_string(),
                    status_code: None,
                    err: Some(HttpAccessorError {
                        err_type: HttpAccessorErrorType::OpenUrlError(e),
                        msg: "async get fail".to_string()
                    })
                })?;
            HttpAccessor::handle_async_resp(url, resp).await
        }
    }

    pub async fn async_multi_get(urls: &[&str], timeout: u32) -> Result<Vec<Result<HttpAccessorResponse, HttpAccessorResponseError>>, HttpAccessorError> {
        let resp_future_list = futures::stream::iter(
            urls.iter().map(|url| {
                async move {
                    HttpAccessor::async_get(url, timeout).await
                }
            })
        ).buffered(128).collect::<Vec<Result<HttpAccessorResponse, HttpAccessorResponseError>>>();
        let resp_wrapper_list = resp_future_list.await;
        Ok(resp_wrapper_list)
    }

    pub async fn async_multi_get_unordered(urls: &[&str], timeout: u32) -> Result<Vec<Result<HttpAccessorResponse, HttpAccessorResponseError>>, HttpAccessorError> {
        let resp_future_list = futures::stream::iter(
            urls.iter().map(|url| {
                async move {
                    HttpAccessor::async_get(url, timeout).await
                }
            })
        ).buffer_unordered(128).collect::<Vec<Result<HttpAccessorResponse, HttpAccessorResponseError>>>();
        let resp_wrapper_list = resp_future_list.await;
        Ok(resp_wrapper_list)
    }
}
