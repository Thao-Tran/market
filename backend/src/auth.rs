use super::models::{AuthError, PasswordHash, Settings, User, SESSION_DURATION};
use chrono::prelude::*;
use chrono::Duration;
use hmac::{Hmac, Mac};
use jwt::claims::RegisteredClaims;
use jwt::{SignWithKey, VerifyWithKey};
use ring::{digest, pbkdf2};
use sha2::Sha256;
use std::num::NonZeroU32;

const CREDENTIAL_LEN: usize = digest::SHA256_OUTPUT_LEN;
static PBKDF2_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA256;

/// Handles authentication.
#[derive(Clone)]
pub struct Auth {
  db_salt: String,
  pbkdf2_iterations: NonZeroU32,
  key: Hmac<Sha256>,
}

impl Auth {
  pub fn new(settings: Settings) -> Auth {
    Auth {
      db_salt: settings.salt.to_string(),
      pbkdf2_iterations: settings.salt_iterations,
      key: Hmac::new_from_slice(settings.jwt_secret.as_bytes()).unwrap(),
    }
  }

  /// Create a salt using the provided email.
  fn salt(&self, email: &str) -> Vec<u8> {
    let mut salt = Vec::with_capacity(self.db_salt.as_bytes().len() + email.as_bytes().len());
    salt.extend(self.db_salt.as_bytes());
    salt.extend(email.as_bytes());
    salt
  }

  /// Create a new password hash.
  pub fn create_hash(&self, email: &str, password: &str) -> PasswordHash {
    let salt = self.salt(email);
    let mut hash: PasswordHash = [0u8; CREDENTIAL_LEN];
    pbkdf2::derive(
      PBKDF2_ALG,
      self.pbkdf2_iterations,
      &salt,
      password.as_bytes(),
      &mut hash,
    );
    hash
  }

  /// Validate the provided credentials.
  fn validate_user(&self, user: &User, password_attempt: &str) -> Result<(), AuthError> {
    let salt = self.salt(&user.email);
    return pbkdf2::verify(
      PBKDF2_ALG,
      self.pbkdf2_iterations,
      &salt,
      password_attempt.as_bytes(),
      &user.hash,
    )
    .map_err(|_| AuthError::WrongUsernameOrPassword);
  }

  /// Validate and create a token for a user.
  pub fn create_token(&self, user: &User, password_attempt: &str) -> Result<String, AuthError> {
    self.validate_user(user, password_attempt)?;
    let now = Utc::now();
    let claims = RegisteredClaims {
      issued_at: Some(now.timestamp().try_into().unwrap()),
      expiration: Some(
        (now + Duration::minutes(SESSION_DURATION))
          .timestamp()
          .try_into()
          .unwrap(),
      ),
      ..Default::default()
    };
    match claims.sign_with_key(&self.key) {
      Ok(token) => Ok(token),
      Err(error) => Err(AuthError::JwtError(error)),
    }
  }

  /// Verify the provided token is still valid.
  pub fn verify_token(&self, token: &str) -> Result<(), AuthError> {
    let claims_result: Result<RegisteredClaims, jwt::error::Error> =
      token.verify_with_key(&self.key);

    let claims = match claims_result {
      Ok(claims) => claims,
      Err(error) => return Err(AuthError::JwtError(error)),
    };

    if let Some(expiration) = claims.expiration {
      if expiration <= Utc::now().timestamp().try_into().unwrap() {
        return Err(AuthError::ExpiredToken);
      }
    }

    Ok(())
  }
}
