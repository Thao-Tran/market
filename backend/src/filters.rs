use super::auth::Auth;
use super::db::Db;
use super::handlers;
use super::models::{Settings, UserReq};
use warp::Filter;

/// The Users filters combined.
pub fn users(
  settings: Settings,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  users_create(settings.clone())
}

/// The Tokens filters combined.
pub fn tokens(
  settings: Settings,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  tokens_create(settings.clone())
}

/// POST /users
fn users_create(
  settings: Settings,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  warp::path!("users")
    .and(warp::post())
    .and(json_body())
    .and(with_db(settings.clone()))
    .and(with_auth(settings))
    .and_then(handlers::create_user)
}

/// POST /tokens
fn tokens_create(
  settings: Settings,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  warp::path!("tokens")
    .and(warp::post())
    .and(json_body())
    .and(with_db(settings.clone()))
    .and(with_auth(settings))
    .and_then(handlers::create_token)
}

/// Include for protected endpoints.
fn require_token(
  settings: Settings,
) -> impl Filter<Extract = ((),), Error = warp::Rejection> + Clone {
  warp::header::header::<String>("Authorization")
    .and(with_auth(settings))
    .and_then(handlers::verify_token)
}

/// Include for endpoints that use the database.
fn with_db(
  settings: Settings,
) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
  warp::any().map(move || Db::new(settings.clone()).unwrap())
}

/// Include for endpoints that use authentication methods.
fn with_auth(
  settings: Settings,
) -> impl Filter<Extract = (Auth,), Error = std::convert::Infallible> + Clone {
  warp::any().map(move || Auth::new(settings.clone()))
}

/// Include for endpoints that expect a request body.
fn json_body() -> impl Filter<Extract = (UserReq,), Error = warp::Rejection> + Clone {
  warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}
