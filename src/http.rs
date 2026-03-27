use std::sync::Arc;

use futures::FutureExt as _;
use gpui::http_client::{AsyncBody, HttpClient, Response, Url};

pub struct SimpleHttpClient {
    client: reqwest::Client,
    runtime: tokio::runtime::Runtime,
}

impl SimpleHttpClient {
    pub fn new() -> Arc<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        // Build the reqwest client inside the Tokio runtime context
        let client = runtime.block_on(async { reqwest::Client::new() });

        Arc::new(Self { client, runtime })
    }
}

impl HttpClient for SimpleHttpClient {
    fn type_name(&self) -> &'static str {
        "SimpleHttpClient"
    }

    fn user_agent(&self) -> Option<&gpui::http_client::http::HeaderValue> {
        None
    }

    fn proxy(&self) -> Option<&Url> {
        None
    }

    fn send(
        &self,
        req: gpui::http_client::Request<AsyncBody>,
    ) -> futures::future::BoxFuture<'static, anyhow::Result<Response<AsyncBody>>> {
        let (parts, _body) = req.into_parts();
        let url = parts.uri.to_string();
        let method = match parts.method.as_str() {
            "GET" => reqwest::Method::GET,
            "POST" => reqwest::Method::POST,
            "PUT" => reqwest::Method::PUT,
            "DELETE" => reqwest::Method::DELETE,
            "HEAD" => reqwest::Method::HEAD,
            other => reqwest::Method::from_bytes(other.as_bytes()).unwrap_or(reqwest::Method::GET),
        };

        let client = self.client.clone();
        let handle = self.runtime.handle().clone();
        async move {
            let resp = handle
                .spawn(async move { client.request(method, &url).send().await })
                .await??;
            let status = resp.status();
            let bytes = resp.bytes().await?;

            let response = gpui::http_client::http::Response::builder()
                .status(status.as_u16())
                .body(AsyncBody::from(bytes.to_vec()))?;

            Ok(response)
        }
        .boxed()
    }
}
