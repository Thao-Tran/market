use super::models::{PasswordHash, Settings, User};
use rusqlite::{params, Connection, DatabaseName, Result};
use strum_macros::Display;
use uuid::Uuid;

/// SQLite database.
pub struct Db {
  conn: Connection,
}

#[derive(Display, Debug)]
enum Tables {
  #[strum(serialize = "users")]
  Users,
}

impl Db {
  /// Create a new database connection.
  pub fn new(settings: Settings) -> Result<Db> {
    let conn = Connection::open(settings.db_path)?;
    let db = Db { conn };
    db.create_tables()?;

    Ok(db)
  }

  /// Setup tables.
  fn create_tables(&self) -> Result<()> {
    self.conn.execute(
      &format!(
        "CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY, email TEXT NOT NULL UNIQUE, hash BLOB NOT NULL)",
        Tables::Users
      )[..],
      []
    )?;
    Ok(())
  }

  /// Add a new user to the database.
  pub fn create_user(&self, user: &User) -> Result<()> {
    self.conn.execute(
      &format!(
        "INSERT INTO {} (id, email, hash) VALUES (?, ?, ?)",
        Tables::Users
      )[..],
      params![
        &user.id.to_string()[..],
        &user.email[..],
        Vec::from(user.hash)
      ],
    )?;

    Ok(())
  }

  /// Get a user from the database based on the email.
  pub fn get_user(&self, email: &str) -> Result<User> {
    self.conn.query_row(
      &format!(
        "SELECT rowid, id, email FROM {} WHERE email = ?",
        Tables::Users
      )[..],
      params![email],
      |row| {
        let id_str = row.get::<usize, String>(1).unwrap();
        let id = Uuid::parse_str(&id_str[..]).unwrap();
        let email = row.get(2).unwrap();
        let blob = self
          .conn
          .blob_open(
            DatabaseName::Main,
            &Tables::Users.to_string()[..],
            "hash",
            row.get(0).unwrap(),
            false,
          )
          .unwrap();
        let mut hash: PasswordHash = Default::default();
        blob.read_at(&mut hash, 0).unwrap();

        Ok(User { id, email, hash })
      },
    )
  }

  /// Close the database connection. Should only be called after all database interactions are done.
  pub fn close(self) -> Result<()> {
    match self.conn.close() {
      Ok(_) => Ok(()),
      Err(err) => Err(err.1),
    }
  }
}
