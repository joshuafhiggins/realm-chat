use std::net::SocketAddr;

use rand::Rng;
use sha3::{Digest, Sha3_256};
use sha3::digest::Update;
use sqlx::{MySql, Pool, Row};
use tarpc::context::Context;

use crate::types::{AuthUser, ErrorCode, RealmAuth};
use crate::types::ErrorCode::*;

#[derive(Clone)]
pub struct RealmAuthServer {
    pub socket: SocketAddr,
    pub db_pool: Pool<MySql>,
}

//TODO: USERNAME FORMATTING!

impl RealmAuthServer {
    pub fn new(socket: SocketAddr, db_pool: Pool<MySql>) -> RealmAuthServer {
        RealmAuthServer {
            socket,
            db_pool,
        }
    }
    
    pub fn gen_login_code(&self) -> u16 {
        let mut rng = rand::thread_rng();
        let mut login_code: u16 = 0;

        for n in 1..=6 {
            if n == 1 {
                login_code += rng.gen_range(1..=9);
            }
            login_code += rng.gen_range(0..=9) * (10^n);
        }
        
        login_code
    }
    
    pub async fn is_username_taken(&self, username: &str) -> Result<bool, ErrorCode> {
        let result = sqlx::query("SELECT NOT EXISTS (SELECT 1 FROM user WHERE username = ?) AS does_exist")
            .bind(username)
            .fetch_one(&self.db_pool).await;
        
        match result {
            Ok(row) => Ok(row.try_get("does_exist").unwrap()),
            Err(_) => Err(InvalidUsername)
        }
    }
    
    pub async fn is_email_taken(&self, email: &str) -> Result<bool, ErrorCode> {
        let result = sqlx::query("SELECT NOT EXISTS (SELECT 1 FROM user WHERE email = ?) AS does_exist")
            .bind(email)
            .fetch_one(&self.db_pool).await;

        match result {
            Ok(row) => Ok(row.try_get("does_exist").unwrap()),
            Err(_) => Err(InvalidUsername)
        }
    }
    
    pub async fn is_authorized(&self, username: &str, token: &str) -> Result<bool, ErrorCode> {
        let result = sqlx::query("SELECT tokens FROM user WHERE username = ?")
            .bind(username).fetch_one(&self.db_pool).await;

        match result {
            Ok(row) => {
                let token_long: &str = row.try_get("tokens").unwrap();
                let tokens = token_long.split(',').collect::<Vec<&str>>();

                for i in 0..tokens.len() {
                    if tokens.get(i).unwrap() == &token {
                        return Ok(true)
                    }
                }

                Ok(false)
            },
            Err(_) => Err(InvalidUsername),
        }
    }
}

impl RealmAuth for RealmAuthServer {
    async fn test(self, _: Context, name: String) -> String {
        format!("Hello {} auth!", name)
    }

    async fn server_token_validation(self, _: Context, server_token: String, username: String, server_id: String, domain: String, tarpc_port: u16) -> bool {
        let result = sqlx::query("SELECT tokens FROM user WHERE username = ?").bind(username).fetch_one(&self.db_pool).await;

        match result {
            Ok(row) => {
                let token_long: &str = row.try_get("tokens").unwrap();
                let tokens = token_long.split(',').collect::<Vec<&str>>();

                for token in tokens {
                    let hash = Sha3_256::new().chain(format!("{}{}{}{}", token, server_id, domain, tarpc_port)).finalize();
                    if hex::encode(hash) == server_token {
                        return true
                    }
                }
                
                false
            },
            Err(_) => false,
        }
    }

    async fn create_account_flow(self, _: Context, username: String, email: String) -> ErrorCode {
        todo!()
    }

    async fn finish_account_flow(self, _: Context, username: String, login_code: u16, avatar: String) -> Result<String, ErrorCode> {
        todo!()
    }

    async fn create_login_flow(self, _: Context, username: String) -> ErrorCode {
        let result = sqlx::query("UPDATE user SET login_code = ? WHERE username = ?;")
            .bind(self.gen_login_code())
            .bind(username)
            .execute(&self.db_pool).await;
        
        match result {
            Ok(_) => { 
                todo!() //TODO: Emails!
            },
            Err(_) => InvalidUsername
        }
    }

    async fn finish_login_flow(self, _: Context, username: String, login_code: u16) -> Result<String, ErrorCode> {
        todo!()
    }

    async fn change_email_flow(self, _: Context, username: String, new_email: String, token: String) -> ErrorCode {
        todo!()
    }

    async fn finish_change_email_flow(self, _: Context, username: String, token: String, login_code: u16) -> ErrorCode {
        todo!()
    }

    async fn change_username(self, _: Context, username: String, token: String, new_username: String) -> ErrorCode {
        let result = self.is_authorized(&username, &token).await;
        match result {
            Ok(authorized) => {
                if !authorized {
                    return Unauthorized
                }
            }
            Err(error) => return error
        }

        let result = self.is_username_taken(&new_username).await;
        match result {
            Ok(is_taken) => {
                if is_taken {
                    return UsernameTaken
                }
            }
            Err(error) => return error
        }

        let result = sqlx::query("UPDATE user SET username = ? WHERE username = ?")
            .bind(&new_username)
            .bind(&username).execute(&self.db_pool).await;
        match result {
            Ok(_) => NoError,
            Err(_) => Error
        }
    }

    async fn change_avatar(self, _: Context, username: String, token: String, new_avatar: String) -> ErrorCode {
        let result = self.is_authorized(&username, &token).await;
        match result {
            Ok(authorized) => {
                if !authorized {
                    return Unauthorized
                }
            }
            Err(error) => return error
        }

        let result = sqlx::query("UPDATE user SET avatar = ? WHERE username = ?")
            .bind(&new_avatar)
            .bind(&username).execute(&self.db_pool).await;
        match result {
            Ok(_) => NoError,
            Err(_) => Error
        }
    }

    async fn get_all_data(self, _: Context, username: String, token: String) -> Result<AuthUser, ErrorCode> {
        let result = self.is_authorized(&username, &token).await;
        match result {
            Ok(authorized) => {
                if !authorized {
                    return Err(Unauthorized)
                }
            }
            Err(error) => return Err(error)
        }

        let result = sqlx::query("SELECT * FROM user WHERE username = ?")
            .bind(&username).fetch_one(&self.db_pool).await;
        match result {
            Ok(row) => {
                Ok(AuthUser {
                    id: row.try_get("id").unwrap(),
                    username: row.try_get("username").unwrap(),
                    email: row.try_get("email").unwrap(),
                    avatar: row.try_get("avatar").unwrap(),
                    login_code: None,
                    bigtoken: row.try_get("tokens").unwrap(),
                    google_oauth: row.try_get("google_oauth").unwrap(),
                    apple_oauth: row.try_get("apple_oauth").unwrap(),
                    github_oauth: row.try_get("github_oauth").unwrap(),
                    discord_oauth: row.try_get("discord_oauth").unwrap(),
                })
            }
            Err(_) => Err(InvalidUsername)
        }
    }

    async fn sign_out(self, _: Context, username: String, token: String) -> ErrorCode {
        let result = sqlx::query("SELECT tokens FROM user WHERE username = ?")
            .bind(&username).fetch_one(&self.db_pool).await;

        match result {
            Ok(row) => {
                let token_long: &str = row.try_get("tokens").unwrap();
                let mut tokens = token_long.split(',').collect::<Vec<&str>>();
                
                for i in 0..tokens.len() {
                    if tokens.get(i).unwrap().eq(&token.as_str()) {
                        tokens.remove(i);
                        
                        let result = sqlx::query("UPDATE user SET tokens = ? WHERE username = ?")
                            .bind(tokens.join(","))
                            .bind(&username)
                            .execute(&self.db_pool).await;
                        
                        match result {
                            Ok(_) => NoError,
                            Err(_) => Error
                        };
                    }
                }

                Unauthorized
            },
            Err(_) => InvalidUsername,
        }
    }

    async fn get_avatar_for_user(self, _: Context, username: String) -> Result<String, ErrorCode> {
        let result = sqlx::query("SELECT tokens FROM user WHERE username = ?").bind(username).fetch_one(&self.db_pool).await;

        match result {
            Ok(row) => Ok(row.try_get("avatar").unwrap_or("".to_string())),
            Err(_) => Err(InvalidUsername)
        }
    }
}