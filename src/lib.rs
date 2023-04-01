use {
    std::{collections::HashMap},
    tower::util::ServiceExt,
    axum::{Router, routing::any, extract::Host, http::{Request, StatusCode}, body::Body, response::Response},
};

pub struct MultiHostRouter {
    handlers: HashMap<String, Router>,
    fallback: Router,
}

impl MultiHostRouter {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            fallback: Router::new().fallback(any(default_fallback)),
        }
    }

    pub fn host(self, host: &str, router: Router) -> Self {
        let mut handlers = self.handlers;
        handlers.insert(host.to_lowercase(), router);
        
        Self {
            handlers,
            ..self
        }
    }

    pub fn fallback(self, router: Router) -> Self {
        Self {
            fallback: router,
            ..self
        }
    }

    pub fn build(self) -> Router {
        let handlers = self.handlers;

        let multi_host_handler = {
            |Host(hostname): Host, request: Request<Body>| async move {
                let router = match handlers.get(&hostname.to_lowercase()) {
                    Some(v) => v,
                    None => &self.fallback,
                }.clone();
                
                let response = router.oneshot(request).await;
                response
            }
        };

        Router::new().fallback(any(multi_host_handler))
    }
}

async fn default_fallback() -> Response<String> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body("not found.\n".to_owned())
        .unwrap()
}