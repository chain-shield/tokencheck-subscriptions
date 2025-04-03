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
