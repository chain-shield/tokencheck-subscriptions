use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, password_hash::PasswordHasher};

#[derive(PartialEq)]
pub enum UserVerificationOrigin {
    Email,
    OAuth,
}
impl ToString for UserVerificationOrigin {
    fn to_string(&self) -> String {
        match self {
            UserVerificationOrigin::Email => "email".to_string(),
            UserVerificationOrigin::OAuth => "oauth".to_string(),
        }
    }
}

pub fn hash_str(key: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(key.as_bytes(), &salt)
        .unwrap()
        .to_string()
}