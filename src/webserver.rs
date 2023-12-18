use axum::Router;
use axum::routing::get;
use tracing::info;

pub trait RouterExt {
    fn configure_webserver(self) -> Self;
}

impl RouterExt for Router {
    fn configure_webserver(self) -> Self {
        self.route("/", get(|| async {
            info!("Hello, World!");
            "Hello, World!"
        }))
    }
}