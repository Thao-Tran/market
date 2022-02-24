use super::auth::Auth;
use super::db::Db;
use super::models::{
  HandlerError, TokenReq, User, UserReq, LOGGED_IN_COOKIE, SESSION_DURATION, TOKEN_COOKIE,
};
use chrono::Duration;
use jsonapi::api::*;
use jsonapi::jsonapi_model;
use jsonapi::model::*;
use std::convert::Infallible;
use uuid::Uuid;
use warp::http::{Response, StatusCode};

jsonapi_model!(User; "users");
jsonapi_model!(UserReq; "users");
jsonapi_model!(TokenReq; "tokens");

/// Parse a JSONAPI document body.
///
/// Rejects with BadRequest if the parsing fails
pub async fn parse_jsonapi_doc<T: JsonApiModel>(req: DocumentData) -> Result<T, warp::Rejection> {
  match T::from_jsonapi_document(&req) {
    Ok(new_user) => Ok(new_user),
    Err(error) => {
      log::debug!("Failed to parse user from request: {:?}", error);
      Err(warp::reject::custom(HandlerError::BadRequest))
    }
  }
}

/// Create a new user.
///
/// Rejects with a HandlerError::Conflict if a user already exists with the same email.
pub async fn create_user<'a>(
  new_user: UserReq,
  db: Db,
  auth: Auth,
) -> Result<impl warp::Reply, warp::Rejection> {
  let error = match db.get_user(&new_user.email) {
    Ok(_) => {
      log::debug!("Failed to create user: user already exists");
      return Err(warp::reject::custom(HandlerError::Conflict));
    }
    Err(error) => error,
  };

  if error != rusqlite::Error::QueryReturnedNoRows {
    log::debug!("Failed to get user: {:?}", error);
    return Err(warp::reject::custom(HandlerError::Db(error)));
  }

  let hash = auth.create_hash(&new_user.email, &new_user.password);
  let user = User {
    id: Uuid::new_v4(),
    email: new_user.email,
    hash,
  };

  if let Err(error) = db.create_user(&user) {
    log::debug!("Failed to get user: {:?}", error);
    return Err(warp::reject::custom(HandlerError::Db(error)));
  }

  if let Err(error) = db.close() {
    log::debug!("Failed to close db: {:?}", error);
    return Err(warp::reject::custom(HandlerError::Db(error)));
  }

  Ok(warp::reply::with_status(
    warp::reply::json(&user.to_jsonapi_document()),
    StatusCode::CREATED,
  ))
}

/// Create and sign a new JWT token to be used for protected endpoints.
pub async fn create_token<'a>(
  user_attempt: TokenReq,
  db: Db,
  auth: Auth,
) -> Result<impl warp::Reply, warp::Rejection> {
  let user = match db.get_user(&user_attempt.email) {
    Ok(user) => user,
    Err(error) => {
      if error == rusqlite::Error::QueryReturnedNoRows {
        log::debug!("Failed to authorize user: user with email does not exist");
        return Err(warp::reject::custom(HandlerError::NotFound));
      }

      log::debug!("Failed to query database: {:?}", error);
      return Err(warp::reject::custom(HandlerError::Db(error)));
    }
  };

  if let Err(error) = db.close() {
    log::debug!("Failed to close database connection: {:?}", error);
    return Err(warp::reject::custom(HandlerError::Db(error)));
  }

  let token = match auth.create_token(&user, &user_attempt.password) {
    Ok(token) => token,
    Err(error) => {
      log::debug!("Failed to create token: {:?}", error);
      return Err(warp::reject::custom(HandlerError::Auth(error)));
    }
  };

  let max_age = Duration::minutes(SESSION_DURATION).num_seconds();
  let loggedin_cookie = format!(
    "{}={:?}; Max-Age={}; Path=/",
    LOGGED_IN_COOKIE, true, max_age
  );
  let token_cookie = format!(
    "{}={}; Max-Age={}; Path=/; HttpOnly",
    TOKEN_COOKIE, token, max_age
  );

  let response = Response::builder()
    .status(StatusCode::NO_CONTENT)
    .header(warp::http::header::SET_COOKIE, loggedin_cookie)
    .header(warp::http::header::SET_COOKIE, token_cookie)
    .body("")
    .unwrap();
  return Ok(response);
}

/// Clear token cookies.
pub async fn delete_token<'a>() -> Result<impl warp::Reply, warp::Rejection> {
  let loggedin_cookie = format!("{}=; Max-Age=0; Path=/", LOGGED_IN_COOKIE);
  let token_cookie = format!("{}=; Max-Age=0; Path=/; HttpOnly", TOKEN_COOKIE);

  let response = Response::builder()
    .status(StatusCode::NO_CONTENT)
    .header(warp::http::header::SET_COOKIE, loggedin_cookie)
    .header(warp::http::header::SET_COOKIE, token_cookie)
    .body("")
    .unwrap();
  return Ok(response);
}

/// Dummy handler to test token verification.
pub async fn test_token<'a>(user_id: String) -> Result<impl warp::Reply, warp::Rejection> {
  Ok(warp::reply::json(&user_id))
}

/// Verify that an authorization token is still valid.
pub async fn verify_token<'a>(token: String, auth: Auth) -> Result<String, warp::Rejection> {
  match auth.verify_token(&token) {
    Ok(user_id) => Ok(user_id),
    Err(error) => {
      log::debug!("Failed to verify token: {:?}", error);
      Err(warp::reject::custom(HandlerError::Auth(error)))
    }
  }
}

/// Rejection handler.
pub async fn rejection(err: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
  let (mut status, title, mut headers) = match err.find() {
    Some(HandlerError::NotFound) => (StatusCode::NOT_FOUND, "Resource not found", vec![]),
    Some(HandlerError::BadRequest) => (StatusCode::BAD_REQUEST, "Request is invalid", vec![]),
    Some(HandlerError::Conflict) => (StatusCode::CONFLICT, "Resource already exists", vec![]),
    Some(HandlerError::Db(_)) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      "Database failure",
      vec![],
    ),
    Some(HandlerError::Auth(_)) => (
      StatusCode::UNAUTHORIZED,
      "Authorization failure",
      // Clear token and logged in cookies.
      vec![
        (
          warp::http::header::SET_COOKIE,
          format!("{}=; Max-Age=0; Path=/", TOKEN_COOKIE),
        ),
        (
          warp::http::header::SET_COOKIE,
          format!("{}=; Max-Age=0; Path=/", LOGGED_IN_COOKIE),
        ),
      ],
    ),
    _ => {
      log::debug!("Unhandled error occurred: {:?}", err);

      if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Resource not found", vec![])
      } else {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          "Unhandled rejection",
          vec![],
        )
      }
    }
  };

  let body = JsonApiDocument::Error(DocumentError {
    errors: vec![JsonApiError {
      title: Some(title.to_string()),
      ..Default::default()
    }],
    ..Default::default()
  });

  let ser_body = match serde_json::to_string(&body) {
    Ok(ser_body) => ser_body,
    Err(error) => {
      log::debug!(
        "Failed to serialize response body in rejection handler: {:?}",
        error
      );
      status = StatusCode::INTERNAL_SERVER_ERROR;
      headers = vec![];
      "".to_string()
    }
  };

  let mut builder = Response::builder().status(status);

  for (key, value) in headers {
    builder = builder.header(key, value);
  }

  let response = builder.body(ser_body).unwrap();

  Ok(response)
}
