use axum::{http::Request, middleware::Next, response::IntoResponse};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use tracing::info;
use uuid::Uuid;

pub const COOKIE_NAME: &str = "ava_device_id";

pub async fn layer_auth<B>(
    jar: CookieJar,
    request: Request<B>,
    next: Next<B>,
) -> impl IntoResponse {
    info!("layer_auth");

    let jar = match jar.get(COOKIE_NAME) {
        Some(_) => jar,
        None => {
            let device_id = Uuid::new_v4().to_string();
            let cookie = Cookie::build(COOKIE_NAME, device_id)
                .path("/")
                .permanent()
                .finish();
            jar.add(cookie)
        }
    };

    (jar, next.run(request).await)
}
