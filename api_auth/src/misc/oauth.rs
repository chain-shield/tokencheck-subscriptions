use std::fmt;

use common::error::{AppError, Res};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OAuthProvider {
    GitHub,
    Google,
    Facebook,
    Apple,
    X,
}
impl OAuthProvider {
    /// Returns the OAuth provider as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            OAuthProvider::GitHub => "github",
            OAuthProvider::Google => "google",
            OAuthProvider::Facebook => "facebook",
            OAuthProvider::Apple => "apple",
            OAuthProvider::X => "x",
        }
    }

    /// Creates an OAuth provider from a string.
    pub fn from_str(s: &str) -> Res<Self> {
        match s {
            "github" => Ok(OAuthProvider::GitHub),
            "google" => Ok(OAuthProvider::Google),
            "facebook" => Ok(OAuthProvider::Facebook),
            "apple" => Ok(OAuthProvider::Apple),
            "x" => Ok(OAuthProvider::X),
            ps => Err(AppError::Internal(format!(
                "Invalid OAuth provider: {}",
                ps
            ))),
        }
    }

    /// Returns the scopes for the OAuth provider.
    pub fn get_scopes(&self) -> Vec<&'static str> {
        match self {
            OAuthProvider::GitHub => vec!["user:email"],
            OAuthProvider::Google => vec!["email profile"],
            OAuthProvider::Facebook => vec!["email"],
            OAuthProvider::Apple => vec!["name email"],
            OAuthProvider::X => vec!["email"],
        }
    }
}
impl fmt::Display for OAuthProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
