use std::{net::SocketAddr, sync::Arc};

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{header::CONTENT_TYPE, Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post},
    Router,
};

use crate::kvs::concurrency::ConcurrencyKvs;
use crate::kvs::Kvs;

// todo: should it to be as a builder?
pub struct AppBuilder;

impl AppBuilder {
    pub fn new<T>(kvs: T) -> AppBuilderBuf<T> {
        AppBuilderBuf {
            kvs,
            root: String::from("/"),
        }
    }
}

pub struct AppBuilderBuf<T> {
    kvs: T,
    root: String,
}

impl<T: Sync + Send + Kvs + 'static> AppBuilderBuf<T> {
    pub async fn run(self, addr: impl Into<SocketAddr>) {
        let (kvs, mut runner) = ConcurrencyKvs::new(self.kvs, 32);
        let k = Arc::new(kvs);
        let handler = runner.spawn();

        // use wildcard `*key` to match a path-like key such as '/foo/bar/key.html'.
        // https://docs.rs/axum/latest/axum/struct.Router.html#wildcards
        let route = format!("{}*key", self.root);
        let app = Router::new()
            .route(
                &route,
                get(get_value).layer(middleware::from_fn(attach_content_type)),
            )
            .route(&route, post(set_value))
            .route(&route, delete(remove_value))
            .with_state(k);
        let server = axum::Server::bind(&addr.into()).serve(app.into_make_service());

        let (_, server_res) = tokio::join!(handler, server);
        server_res.unwrap()
    }
}

async fn get_value(
    Path(key): Path<String>,
    State(state): State<Arc<ConcurrencyKvs>>,
) -> Result<Bytes, StatusCode> {
    match state.make_handle().get(key.into()).await {
        Ok(value) => match value {
            Some(value) => Ok(value.into()),
            None => Err(StatusCode::NOT_FOUND),
        },
        Err(_) => {
            // todo: log error.
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn set_value(
    Path(key): Path<String>,
    State(state): State<Arc<ConcurrencyKvs>>,
    bytes: Bytes,
) -> Result<(), StatusCode> {
    match state.make_handle().insert(key.into(), bytes.into()).await {
        Ok(_) => Ok(()),
        Err(_) => {
            // todo: log error.
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn remove_value(
    Path(key): Path<String>,
    State(state): State<Arc<ConcurrencyKvs>>,
) -> Result<(), StatusCode> {
    match state.make_handle().remove(key.into()).await {
        Ok(_) => Ok(()),
        Err(_) => {
            // todo: log error.
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Attaches a content-type that detected from request uri.
///
/// - If request uri is `/foo.html`, attaches content-type as `text/html`.
/// - If could not detect a content-type, attaches content-type as `application/octet-stream`.
async fn attach_content_type<B>(request: Request<B>, next: Next<B>) -> Response {
    let conetnt_type =
        mime_db::lookup(request.uri().to_string()).unwrap_or("application/octet-stream");
    let mut res = next.run(request).await;
    res.headers_mut()
        .insert(CONTENT_TYPE, conetnt_type.parse().unwrap());
    res
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{header::CONTENT_TYPE, Request},
        middleware::from_fn,
        routing::get,
        Router,
    };

    use crate::http::attach_content_type;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_attach_content_type() {
        async fn handle() {}

        let app = Router::new()
            .route("/", get(handle))
            .layer(from_fn(attach_content_type));

        let res = app
            .oneshot(
                Request::builder()
                    .uri("/index.html")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            res.headers().get(CONTENT_TYPE),
            Some(&"text/html".parse().unwrap())
        )
    }
}
