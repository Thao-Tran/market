use ring::digest;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use uuid::Uuid;

const HASH_LEN: usize = digest::SHA256_OUTPUT_LEN;
pub type PasswordHash = [u8; HASH_LEN];

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserReq {
  pub id: String, // Actually not used but jsonapi doesn't allow partial resources yet
  pub email: String,
  pub password: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct User {
  pub id: Uuid,
  pub email: String,
  pub hash: PasswordHash,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Token {
  pub id: Uuid,
  pub token: String,
}

#[derive(Deserialize, Clone)]
pub struct Settings {
  pub salt_iterations: NonZeroU32,
  pub salt: String,
  pub jwt_secret: String,
  pub db_path: String,
}

#[derive(Debug)]
pub enum AuthError {
  JwtError(jwt::error::Error),
  WrongUsernameOrPassword,
  ExpiredToken,
  InvalidToken,
}

impl warp::reject::Reject for AuthError {}

#[derive(Debug)]
pub enum HandlerError {
  Auth(AuthError),
  Db(rusqlite::Error),
  Conflict,
  BadRequest,
}

impl warp::reject::Reject for HandlerError {}
