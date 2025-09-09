use sqlx::{sqlite::SqlitePool, migrate::MigrateDatabase, Sqlite, Row};
use crate::models::{User, RegisterForm};
use bcrypt::{hash, verify, DEFAULT_COST};
use anyhow::Result;

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let database_url = "sqlite://users.db";
        
        // Create database if it doesn't exist
        if !Sqlite::database_exists(database_url).await.unwrap_or(false) {
            println!("Creating database {}", database_url);
            Sqlite::create_database(database_url).await?;
        }

        let pool = SqlitePool::connect(database_url).await?;
        
        // Create tables if they don't exist (simple migration)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY NOT NULL,
                email TEXT UNIQUE NOT NULL,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                first_name TEXT,
                last_name TEXT,
                avatar_url TEXT,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                created_at DATETIME NOT NULL DEFAULT (datetime('now')),
                updated_at DATETIME NOT NULL DEFAULT (datetime('now'))
            )
            "#
        )
        .execute(&pool)
        .await?;
        
        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)")
            .execute(&pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)")
            .execute(&pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_active ON users(is_active)")
            .execute(&pool)
            .await?;
        
        Ok(Self { pool })
    }

    pub async fn create_user(&self, form: RegisterForm) -> Result<User> {
        let password_hash = hash(&form.password, DEFAULT_COST)?;
        
        let user = User::new(form.email, form.username, password_hash);
        
        sqlx::query(
            "INSERT INTO users (id, email, username, password_hash, first_name, last_name, is_active, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
        )
        .bind(&user.id)
        .bind(&user.email)
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(&form.first_name)
        .bind(&form.last_name)
        .bind(user.is_active)
        .bind(user.created_at)
        .bind(user.updated_at)
        .execute(&self.pool)
        .await?;
        
        Ok(user)
    }

    pub async fn authenticate_user(&self, email: &str, password: &str) -> Result<Option<User>> {
        let row = sqlx::query(
            "SELECT id, email, username, password_hash, first_name, last_name, avatar_url, is_active, created_at, updated_at FROM users WHERE email = ?1 AND is_active = 1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(record) = row {
            let password_hash: String = record.get("password_hash");
            let is_valid = verify(password, &password_hash)?;
            
            if is_valid {
                let user = User {
                    id: record.get("id"),
                    email: record.get("email"),
                    username: record.get("username"),
                    password_hash,
                    first_name: record.get("first_name"),
                    last_name: record.get("last_name"),
                    avatar_url: record.get("avatar_url"),
                    is_active: record.get::<i64, _>("is_active") != 0,
                    created_at: record.get("created_at"),
                    updated_at: record.get("updated_at"),
                };
                return Ok(Some(user));
            }
        }
        
        Ok(None)
    }

    pub async fn email_exists(&self, email: &str) -> Result<bool> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM users WHERE email = ?1")
            .bind(email)
            .fetch_one(&self.pool)
            .await?;
        
        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    pub async fn username_exists(&self, username: &str) -> Result<bool> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM users WHERE username = ?1")
            .bind(username)
            .fetch_one(&self.pool)
            .await?;
        
        let count: i64 = row.get("count");
        Ok(count > 0)
    }

}