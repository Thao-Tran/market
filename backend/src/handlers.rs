use super::auth::Auth;
use super::db::Db;
use super::models::{HandlerError, Token, User, UserReq};
use jsonapi::api::*;
use jsonapi::jsonapi_model;
use jsonapi::model::*;
use std::convert::Infallible;
use uuid::Uuid;
use warp::http::{Response, StatusCode};

jsonapi_model!(User; "users");
jsonapi_model!(UserReq; "users");
jsonapi_model!(Token; "tokens");

/// Parse a User request body from a JSONAPI document body.
///
/// Rejects with a HandlerError::Conflict if a user already exists with the same email.
pub async fn parse_user_req<'a>(req: DocumentData) -> Result<UserReq, warp::Rejection> {
  match UserReq::from_jsonapi_document(&req) {
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
  user_attempt: UserReq,
  db: Db,
  auth: Auth,
) -> Result<impl warp::Reply, warp::Rejection> {
  let user = match db.get_user(&user_attempt.email) {
    Ok(user) => user,
    Err(error) => {
      if error == rusqlite::Error::QueryReturnedNoRows {
        log::debug!("Failed to authorize user: user with email does not exist");
        return Err(warp::reject::not_found());
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

  let token_cookie = format!("token={:?}; HttpOnly", token.token);

  let response = Response::builder()
    .status(StatusCode::NO_CONTENT)
    .header(warp::http::header::SET_COOKIE, token_cookie)
    .body("")
    .unwrap();
  return Ok(response);
}

/// Verify that an authorization token is still valid.
pub async fn verify_token<'a>(token: String, auth: Auth) -> Result<(), warp::Rejection> {
  if let Err(error) = auth.verify_token(&token) {
    log::debug!("Failed to verify token: {:?}", error);
    return Err(warp::reject::custom(HandlerError::Auth(error)));
  }

  Ok(())
}

/// Rejection handler.
pub async fn rejection(err: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
  let status;
  let title;

  if err.is_not_found() {
    status = StatusCode::NOT_FOUND;
    title = "Resource not found";
  } else if let Some(HandlerError::Auth(_)) = err.find() {
    status = StatusCode::UNAUTHORIZED;
    title = "Authorization failure";
  } else if let Some(HandlerError::BadRequest) = err.find() {
    status = StatusCode::BAD_REQUEST;
    title = "Request is invalid";
  } else if let Some(HandlerError::Conflict) = err.find() {
    status = StatusCode::CONFLICT;
    title = "Resource already exists";
  } else if let Some(HandlerError::Db(_)) = err.find() {
    status = StatusCode::INTERNAL_SERVER_ERROR;
    title = "Database failure";
  } else {
    status = StatusCode::INTERNAL_SERVER_ERROR;
    title = "Unhandled rejection";
  }

  let body = JsonApiDocument::Error(DocumentError {
    errors: vec![JsonApiError {
      title: Some(title.to_string()),
      ..Default::default()
    }],
    ..Default::default()
  });

  Ok(warp::reply::with_status(warp::reply::json(&body), status))
}
