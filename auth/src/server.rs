use std::net::SocketAddr;
use chrono::Utc;
use mail_send::{Credentials, SmtpClientBuilder};
use mail_send::mail_builder::MessageBuilder;
use rand::Rng;
use sha3::{Digest, Sha3_256};
use sha3::digest::Update;
use sqlx::{MySql, Pool, Row};
use sqlx::mysql::{MySqlQueryResult, MySqlRow};
use tarpc::context::Context;

use crate::types::{AuthEmail, AuthUser, ErrorCode, RealmAuth};
use crate::types::ErrorCode::*;

#[derive(Clone)]
pub struct RealmAuthServer {
    pub socket: SocketAddr,
    pub db_pool: Pool<MySql>,
    pub auth_email: AuthEmail,
    pub template_html: String,
    pub template_txt: String,
}

impl RealmAuthServer {
    pub fn new(socket: SocketAddr, db_pool: Pool<MySql>, auth_email: AuthEmail) -> RealmAuthServer {
        RealmAuthServer {
            socket,
            db_pool,
            auth_email,
            template_html: std::fs::read_to_string("./login_email.html").expect("A login_email.html file is needed"),
            template_txt: std::fs::read_to_string("./login_email.txt").expect("A login_email.txt file is needed"),
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
    
    pub async fn send_login_message(&self, username: &str, email: &str, login_code: u16) -> ErrorCode {
        let message = MessageBuilder::new()
            .from((self.auth_email.auth_name.clone(), self.auth_email.auth_username.clone()))
            .to(vec![
                (username, email),
            ])
            .subject(format!("Realm confirmation code: {}", &login_code))
            .html_body(self.template_html.replace("{}", &login_code.to_string()))
            .text_body(self.template_txt.replace("{}", &login_code.to_string()));
        
        let result = SmtpClientBuilder::new(&self.auth_email.server_address, self.auth_email.server_port)
            .implicit_tls(false)
            .credentials(Credentials::new(&self.auth_email.auth_username, &self.auth_email.auth_password))
            .connect()
            .await;
        
        match result {
            Ok(mut client) => {
                let result = client.send(message).await;
                match result {
                    Ok(_) => {
                        NoError
                    }
                    Err(_) => {
                        UnableToSendMail
                    }
                }
            }
            Err(_) => {
                UnableToConnectToMail
            }
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
        //TODO: USERNAME FORMATTING!
        
        

        let result = self.is_username_taken(&username).await;
        match result {
            Ok(taken) => {
                if taken {
                    return UsernameTaken
                }
            }
            Err(error) => return error
        }
        
        let result = self.is_email_taken(&email).await;
        match result {
            Ok(taken) => {
                if taken {
                    return EmailTaken
                }
            }
            Err(error) => return error
        }
        
        let code = self.gen_login_code();
        let result = self.send_login_message(&username, &email, code).await;
        
        if result != NoError {
            return result;
        }
        
        let result = sqlx::query("INSERT INTO user (username, email, avatar, login_code, tokens) VALUES (?, ?, '', ?, '')")
            .bind(&username).bind(&email).bind(code).execute(&self.db_pool).await;
        
        match result {
            Ok(_) => NoError,
            Err(_) => Error
        }
    }

    async fn create_login_flow(self, _: Context, mut username: Option<String>, mut email: Option<String>) -> ErrorCode {
        if username.is_none() && email.is_none() {
            return Error
        }
        
        if username.is_none() {
            let result = sqlx::query("SELECT username FROM user WHERE email = ?;")
                .bind(&email.clone().unwrap())
                .fetch_one(&self.db_pool).await;
            
            match result {
                Ok(row) => {
                    username = row.try_get("username").unwrap();
                }
                Err(_) => return InvalidEmail
            }
        }

        if email.is_none() {
            let result = sqlx::query("SELECT email FROM user WHERE username = ?;")
                .bind(&username.clone().unwrap())
                .fetch_one(&self.db_pool).await;

            match result {
                Ok(row) => {
                    email = row.try_get("email").unwrap();
                }
                Err(_) => return InvalidUsername
            }
        }
        
        let code = self.gen_login_code();
        
        let result = sqlx::query("UPDATE user SET login_code = ? WHERE username = ?;")
            .bind(code)
            .bind(&username)
            .execute(&self.db_pool).await;
        
        match result {
            Ok(_) => self.send_login_message(&username.unwrap(), &email.unwrap(), code).await,
            Err(_) => InvalidUsername
        }
    }

    async fn finish_login_flow(self, _: Context, username: String, login_code: u16) -> Result<String, ErrorCode> {
        let result = sqlx::query("SELECT login_code FROM user WHERE username = ?;")
            .bind(&username)
            .fetch_one(&self.db_pool).await;
        
        match result {
            Ok(row) => {
                if row.try_get::<u16, _>("login_code").unwrap() != login_code {
                    return Err(InvalidLoginCode)
                }
            }
            Err(_) => return Err(InvalidUsername)
        }
        
        let _ = sqlx::query("UPDATE user SET login_code = NULL WHERE username = ?").bind(&username).execute(&self.db_pool).await;

        let hash = Sha3_256::new().chain(format!("{}{}{}", username, login_code, Utc::now().to_utc())).finalize();
        let token = hex::encode(hash);

        let result = sqlx::query("SELECT tokens FROM user WHERE username = ?").bind(&username).fetch_one(&self.db_pool).await;
        match result {
            Ok(row) => {
                let token_long: &str = row.try_get("tokens").unwrap();
                let mut tokens = token_long.split(',').collect::<Vec<&str>>();
                tokens.push(&token);

                let result = sqlx::query("UPDATE user SET tokens = ? WHERE username = ?").bind(tokens.join(",")).bind(&username).execute(&self.db_pool).await;
                match result {
                    Ok(_) => Ok(token),
                    Err(_) => Err(InvalidUsername)
                }
            }
            Err(_) => Err(InvalidUsername)
        }
    }

    async fn change_email_flow(self, _: Context, username: String, new_email: String, token: String) -> ErrorCode {
        todo!()
    }

    async fn finish_change_email_flow(self, _: Context, username: String, token: String, login_code: u16) -> ErrorCode {
        todo!()
    }

    async fn change_username(self, _: Context, username: String, token: String, new_username: String) -> ErrorCode {
        //TODO: USERNAME FORMATTING!


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