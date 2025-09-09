use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(email: String, username: String, password_hash: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            email,
            username,
            password_hash,
            first_name: None,
            last_name: None,
            avatar_url: None,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn full_name(&self) -> String {
        match (&self.first_name, &self.last_name) {
            (Some(first), Some(last)) => format!("{} {}", first, last),
            (Some(first), None) => first.clone(),
            (None, Some(last)) => last.clone(),
            (None, None) => self.username.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionUser {
    pub id: String,
    pub email: String,
    pub username: String,
    pub full_name: String,
    pub avatar_url: Option<String>,
}

impl From<User> for SessionUser {
    fn from(user: User) -> Self {
        let full_name = user.full_name();
        Self {
            id: user.id,
            email: user.email,
            username: user.username,
            full_name,
            avatar_url: user.avatar_url,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
    pub remember_me: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterForm {
    pub email: String,
    pub username: String,
    pub password: String,
    pub password_confirm: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

impl RegisterForm {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.email.is_empty() {
            errors.push("Email is required".to_string());
        } else if !self.email.contains('@') {
            errors.push("Please enter a valid email address".to_string());
        }

        if self.username.is_empty() {
            errors.push("Username is required".to_string());
        } else if self.username.len() < 3 {
            errors.push("Username must be at least 3 characters long".to_string());
        }

        if self.password.is_empty() {
            errors.push("Password is required".to_string());
        } else if self.password.len() < 6 {
            errors.push("Password must be at least 6 characters long".to_string());
        }

        if self.password != self.password_confirm {
            errors.push("Passwords do not match".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}