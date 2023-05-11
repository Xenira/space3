use crate::diesel::ExpressionMethods;
use crate::diesel::RunQueryDsl;
use crate::{schema::users, Database};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use chrono::{NaiveDateTime, Utc};
use diesel::{insert_into, QueryDsl};
use hmac::{Hmac, Mac};
use jwt::Claims;
use jwt::RegisteredClaims;
use jwt::SignWithKey;
use rand_core::OsRng;
use rocket::serde::{json::Json, Deserialize};
use sha2::Sha256;

// TODO: Reduce expiration time and add refresh token support
const JWT_EXPIRATION: u64 = 12 * 60_000;

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub salt: String,
    pub currency: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub salt: String,
}

#[derive(Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Credentials {
    fn to_new_user(&self) -> Result<NewUser, ()> {
        let salt = SaltString::generate(&mut OsRng);
        let username: String = self.username.clone();

        // Hash password to PHC string ($argon2id$v=19$...)
        if let Ok(password_hash) = self.hash_password(&salt) {
            return Ok(NewUser {
                username,
                password: password_hash,
                salt: salt.to_string(),
            });
        }
        Err(())
    }

    fn hash_password(&self, salt: &SaltString) -> Result<String, ()> {
        // Argon2 with default params (Argon2id v19)
        // Hash password to PHC string ($argon2id$v=19$...)
        if let Ok(hash) = Argon2::default().hash_password(self.password.as_bytes(), &salt) {
            return Ok(hash.to_string());
        }

        Err(())
    }
}

#[put("/users/register", data = "<creds>")]
pub async fn register(creds: Json<Credentials>, db: Database) {
    let new_user = creds.to_new_user().expect("Failed to hash password");

    db.run(|con| insert_into(users::table).values(new_user).execute(con))
        .await
        .expect("Failed to create user");
}

#[post("/users/login", data = "<creds>")]
pub async fn login(creds: Json<Credentials>, db: Database) -> String {
    let username = creds.username.clone();
    let user = db
        .run(move |con| {
            users::table
                .filter(users::username.eq(username))
                .first::<User>(con)
        })
        .await
        .expect("Failed to retrieve user from db");

    assert_eq!(
        creds
            .hash_password(&SaltString::new(user.salt.as_str()).expect("User salt currupted"))
            .expect("Failed to verify login")
            .to_string(),
        user.password
    );

    let key: Hmac<Sha256> =
        Hmac::new_from_slice(b"some-secret").expect("Could not instanciate jwt secret");
    let mut claims = Claims::new(RegisteredClaims {
        subject: Some(user.username),
        expiration: Some(Utc::now().timestamp_millis() as u64 + JWT_EXPIRATION),
        ..Default::default()
    });
    claims.sign_with_key(&key).expect("Failed to generate jwt")
}
