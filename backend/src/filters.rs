use super::auth::Auth;
use super::db::Db;
use super::handlers;
use super::models::{Settings, TokenReq, UserReq, TOKEN_COOKIE};
use jsonapi::model::JsonApiModel;
use warp::http::{header, Method};
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
    .or(tokens_test(settings.clone()))
    .or(tokens_delete())
}

/// POST /users
fn users_create(
  settings: Settings,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  warp::path!("users")
    .and(warp::post())
    .and(with_jsonapi_doc::<UserReq>())
    .and(with_db(settings.clone()))
    .and(with_auth(settings.clone()))
    .and_then(handlers::create_user)
}

/// POST /tokens
fn tokens_create(
  settings: Settings,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  warp::path!("tokens")
    .and(warp::post())
    .and(with_jsonapi_doc::<TokenReq>())
    .and(with_db(settings.clone()))
    .and(with_auth(settings.clone()))
    .and_then(handlers::create_token)
}

/// DELETE /tokens
fn tokens_delete() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  warp::path!("tokens")
    .and(warp::delete())
    .and_then(handlers::delete_token)
}

/// GET /tokens
///
/// Dummy filter to test token verification.
fn tokens_test(
  settings: Settings,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  warp::path!("tokens")
    .and(warp::get())
    .and(require_token(settings.clone()))
    .and_then(handlers::test_token)
}

/// Include for protected endpoints.
fn require_token(
  settings: Settings,
) -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
  warp::cookie::<String>(TOKEN_COOKIE)
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

// Include for endpoints that expect a JSON:API request body.
fn with_jsonapi_doc<T: JsonApiModel>(
) -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
  warp::header::exact_ignore_case("content-type", "application/vnd.api+json")
    .and(warp::body::content_length_limit(1024 * 16))
    .and(warp::body::bytes())
    .map(|bytes: warp::hyper::body::Bytes| serde_json::from_slice(&bytes.to_vec()).unwrap())
    .and_then(handlers::parse_jsonapi_doc::<T>)
}

/// Include CORS.
pub fn with_cors(settings: Settings) -> warp::cors::Cors {
  warp::cors()
    .allow_credentials(true)
    .allow_origin(&settings.frontend_host[..])
    .allow_methods(vec![
      Method::OPTIONS,
      Method::POST,
      Method::GET,
      Method::DELETE,
    ])
    .allow_header(header::CONTENT_TYPE)
    .expose_header(header::SET_COOKIE)
    .build()
}
