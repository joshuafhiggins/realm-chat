use std::env;
use std::net::SocketAddr;

use chrono::Utc;
use mail_send::{Credentials, SmtpClientBuilder};
use mail_send::mail_builder::MessageBuilder;
use rand::Rng;
use regex::Regex;
use sha3::{Digest, Sha3_256};
use sha3::digest::Update;
use sqlx::{Pool, query, Sqlite};
use tarpc::context::Context;
use tracing::*;

use crate::types::{AuthEmail, AuthUser, RealmAuth};
use realm_shared::types::ErrorCode;
use realm_shared::types::ErrorCode::*;

#[derive(Clone)]
pub struct RealmAuthServer {
    pub socket: SocketAddr,
    pub db_pool: Pool<Sqlite>,
    pub auth_email: AuthEmail,
    pub template_html: String,
    pub template_txt: String,
    pub domain: String,
}

impl RealmAuthServer {
    pub fn new(socket: SocketAddr, db_pool: Pool<Sqlite>, auth_email: AuthEmail) -> RealmAuthServer {
        RealmAuthServer {
            socket,
            db_pool,
            auth_email,
            template_html: std::fs::read_to_string("./login_email.html").expect("A login_email.html file is needed"),
            template_txt: std::fs::read_to_string("./login_email.txt").expect("A login_email.txt file is needed"),
            domain: env::var("DOMAIN").expect("DOMAIN must be set"),
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
        let result = query!("SELECT NOT EXISTS (SELECT 1 FROM user WHERE username = ?) AS does_exist", username).fetch_one(&self.db_pool).await;
        
        match result {
            Ok(row) => Ok(row.does_exist.unwrap() != 0),
            Err(_) => Err(InvalidUsername)
        }
    }
    
    pub async fn is_email_taken(&self, email: &str) -> Result<bool, ErrorCode> {
        let result = query!("SELECT NOT EXISTS (SELECT 1 FROM user WHERE email = ?) AS does_exist", email).fetch_one(&self.db_pool).await;

        match result {
            Ok(row) => Ok(row.does_exist.unwrap() != 0),
            Err(_) => Err(InvalidUsername)
        }
    }
    
    pub async fn is_authorized(&self, username: &str, token: &str) -> Result<bool, ErrorCode> {
        let result = query!("SELECT tokens FROM user WHERE username = ?", username).fetch_one(&self.db_pool).await;

        match result {
            Ok(row) => {
                let token_long: &str = &row.tokens.unwrap();
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
    
    pub async fn send_login_message(&self, username: &str, email: &str, login_code: u16) -> Result<(), ErrorCode> {
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
                        Ok(())
                    }
                    Err(_) => {
                        Err(UnableToSendMail)
                    }
                }
            }
            Err(_) => {
                Err(UnableToConnectToMail)
            }
        }
    }

    pub async fn is_login_code_valid(&self, username: &str, login_code: u16) -> Result<bool, ErrorCode> {
        let result = query!("SELECT login_code FROM user WHERE username = ?;", username).fetch_one(&self.db_pool).await;

        match result {
            Ok(row) => {
                if row.login_code.unwrap() as u16 != login_code {
                    return Ok(false)
                }
                Ok(true)
            }
            Err(_) => Err(InvalidUsername)
        }
    }

    pub fn is_username_valid(&self, username: &str) -> bool {
        if !username.starts_with('@') || !username.contains(':') {
            return false
        }

        let name = &username[1..username.find(':').unwrap()];
        let domain = &username[username.find(':').unwrap()+1..];

        let re = Regex::new(r"^[a-zA-Z0-9]+$").unwrap();
        if !re.is_match(name) {
            return false
        }
        
        if !domain.eq(&self.domain) {
            return false
        }

        true
    }
}

impl RealmAuth for RealmAuthServer {
    async fn test(self, _: Context, name: String) -> String {
        format!("Hello {} auth!", name)
    }

    async fn server_token_validation(self, _: Context, server_token: String, username: String, server_id: String, domain: String, tarpc_port: u16) -> bool {
        info!("API Request: server_token_validation( server_token -> {}, username -> {}, server_id -> {}, domain -> {}, tarpc_port -> {} )", 
            server_token, username, server_id, domain, tarpc_port);

        let result = query!("SELECT tokens FROM user WHERE username = ?", username).fetch_one(&self.db_pool).await;

        match result {
            Ok(row) => {
                let token_long: &str = &row.tokens.unwrap();
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

    async fn create_account_flow(self, _: Context, username: String, email: String) -> Result<(), ErrorCode> {
        info!("API Request: create_account_flow( username -> {}, email -> {} )", username, email);

        if !self.is_username_valid(&username) {
            return Err(InvalidUsername)
        }
        
        if self.is_username_taken(&username).await? {
            return Err(UsernameTaken)
        }

        if self.is_email_taken(&email).await? {
            return Err(EmailTaken)
        }
        
        let code = self.gen_login_code();
        self.send_login_message(&username, &email, code).await?;
        
        let result = query!("INSERT INTO user (username, email, avatar, login_code, tokens) VALUES (?, ?, '', ?, '')", username, email, code)
            .execute(&self.db_pool).await;
        
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(Error)
        }
    }

    async fn create_login_flow(self, _: Context, mut username: Option<String>, mut email: Option<String>) -> Result<(), ErrorCode> {
        info!("API Request: create_login_flow( username -> {}, email -> {} )", username.clone().unwrap_or("None".to_string()), email.clone().unwrap_or("None".to_string()));

        if username.is_none() && email.is_none() {
            return Err(Error)
        }
        
        if username.is_none() {
            let tmp = email.clone().unwrap();
            let result = query!("SELECT username FROM user WHERE email = ?;", tmp)
                .fetch_one(&self.db_pool).await;
            
            match result {
                Ok(row) => {
                    username = Some(row.username);
                }
                Err(_) => return Err(InvalidEmail)
            }
        }

        if email.clone().is_none() {
            let tmp = username.clone().unwrap();
            let result = query!("SELECT email FROM user WHERE username = ?;", tmp)
                .fetch_one(&self.db_pool).await;

            match result {
                Ok(row) => {
                    email = Some(row.email);
                }
                Err(_) => return Err(InvalidUsername)
            }
        }
        
        let code = self.gen_login_code();
        
        let result = query!("UPDATE user SET login_code = ? WHERE username = ?;", code, username)
            .execute(&self.db_pool).await;
        
        match result {
            Ok(_) => self.send_login_message(&username.unwrap(), &email.unwrap(), code).await,
            Err(_) => Err(InvalidUsername)
        }
    }

    async fn finish_login_flow(self, _: Context, username: String, login_code: u16) -> Result<String, ErrorCode> {
        info!("API Request: finish_login_flow( username -> {}, login_code -> {} )", username, login_code);

        if !self.is_login_code_valid(&username, login_code).await? {
            error!("Unauthorized request made for finish_login_flow() (bad login code)! username -> {}, login_code -> {}", username, login_code);
            return Err(InvalidLoginCode)
        }
        
        let _ = query!("UPDATE user SET login_code = NULL WHERE username = ?", username).execute(&self.db_pool).await;

        let hash = Sha3_256::new().chain(format!("{}{}{}", username, login_code, Utc::now().to_utc())).finalize();
        let token = hex::encode(hash);

        let result = query!("SELECT tokens FROM user WHERE username = ?", username).fetch_one(&self.db_pool).await;
        match result {
            Ok(row) => {
                let token_long: &str = &row.tokens.unwrap();
                let mut tokens = token_long.split(',').collect::<Vec<&str>>();
                tokens.push(&token);

                let mega_token = tokens.join(",");
                let result = query!("UPDATE user SET tokens = ? WHERE username = ?", mega_token, username)
                    .execute(&self.db_pool).await;
                match result {
                    Ok(_) => Ok(token),
                    Err(_) => Err(InvalidUsername)
                }
            }
            Err(_) => Err(InvalidUsername)
        }
    }

    async fn change_email_flow(self, _: Context, username: String, new_email: String, token: String) -> Result<(), ErrorCode> {
        info!("API Request: change_email_flow( username -> {}, new_email -> {}, token -> {} )", username, new_email, token);

        if !self.is_authorized(&username, &token).await? {
            return Err(Unauthorized)
        }

        if self.is_email_taken(&new_email).await? {
            return Err(EmailTaken)
        }

        let _ = query!("UPDATE user SET new_email = ? WHERE username = ?", new_email, username).execute(&self.db_pool).await.unwrap();

        let code = self.gen_login_code();

        let result = query!("UPDATE user SET login_code = ? WHERE username = ?;", code, username)
            .execute(&self.db_pool).await;

        match result {
            Ok(_) => self.send_login_message(&username, &new_email, code).await,
            Err(_) => Err(InvalidUsername)
        }
    }

    async fn finish_change_email_flow(self, _: Context, username: String, new_email: String, token: String, login_code: u16) -> Result<(), ErrorCode> {
        info!("API Request: finish_change_email_flow( username -> {}, new_email -> {}, token -> {}, login_code -> {} )", username, new_email, token, login_code);

        if !self.is_authorized(&username, &token).await? {
            error!("Unauthorized request made for finish_change_email_flow() (bad token)! username -> {}, token -> {}", username, token);
            return Err(Unauthorized)
        }

        if self.is_email_taken(&new_email).await? {
            error!("Email already taken for email change (but its the end of the flow?!) username -> {}, new_email -> {}", username, new_email);
            return Err(EmailTaken)
        }

        if !self.is_login_code_valid(&username, login_code).await? {
            error!("Unauthorized request made for finish_change_email_flow() (bad login code)! username -> {}, login_code -> {}", username, login_code);
            return Err(InvalidLoginCode)
        }

        let _ = query!("UPDATE user SET new_email = NULL WHERE username = ?", username).execute(&self.db_pool).await;

        let _ = query!("UPDATE user SET email = ? WHERE username = ?", new_email, username).execute(&self.db_pool).await;

        Ok(())
    }

    async fn change_username(self, _: Context, username: String, token: String, new_username: String) -> Result<(), ErrorCode> {
        info!("API Request: change_username( username -> {}, token -> {}, new_username -> {} )", username, token, new_username);
        
        if !self.is_authorized(&username, &token).await? {
            error!("Unauthorized request made for change_username()! username -> {}, token -> {}", username, token);
            return Err(Unauthorized)
        }
        
        if !self.is_username_valid(&new_username) {
            error!("Malformed username in request for change_username()! new_username -> {}", new_username);
            return Err(InvalidUsername)
        }

        if self.is_username_taken(&new_username).await? {
            error!("Username is taken for change_username()! new_username -> {}", new_username);
            return Err(UsernameTaken)
        }

        let result = query!("UPDATE user SET username = ? WHERE username = ?", new_username, username).execute(&self.db_pool).await;
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(Error)
        }
    }

    async fn change_avatar(self, _: Context, username: String, token: String, new_avatar: String) -> Result<(), ErrorCode> {
        info!("API Request: change_avatar( username -> {}, token -> {}, new_avatar -> {} )", username, token, new_avatar);
        
        if !self.is_authorized(&username, &token).await? {
            error!("Unauthorized request made for change_avatar()! username -> {}, token -> {}", username, token);
            return Err(Unauthorized)
        }

        let result = query!("UPDATE user SET avatar = ? WHERE username = ?", new_avatar, username).execute(&self.db_pool).await;
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(Error)
        }
    }

    async fn get_all_data(self, _: Context, username: String, token: String) -> Result<AuthUser, ErrorCode> {
        info!("API Request: get_all_data( username -> {}, token -> {} )", username, token);
        
        if !self.is_authorized(&username, &token).await? {
            error!("Unauthorized request made for get_all_data()! username -> {}, token -> {}", username, token);
            return Err(Unauthorized)
        }

        let result = query!(r"SELECT * FROM user WHERE username = ?", username).fetch_one(&self.db_pool).await;
        match result {
            Ok(row) => {
                Ok(AuthUser {
                    id: row.id,
                    username: row.username,
                    email: row.email,
                    avatar: row.avatar,
                    login_code: None,
                    bigtoken: row.tokens,
                    google_oauth: row.google_oauth,
                    apple_oauth: row.apple_oauth,
                    github_oauth: row.github_oauth,
                    discord_oauth: row.discord_oauth,
                })
            }
            Err(_) => {
                error!("Invalid username in request for get_all_data()! username -> {}", username);
                Err(InvalidUsername)
            }
        }
    }

    async fn sign_out(self, _: Context, username: String, token: String) -> Result<(), ErrorCode> {
        info!("API Request: sign_out( username -> {}, token -> {} )", username, token);
        
        let result = query!("SELECT tokens FROM user WHERE username = ?", username).fetch_one(&self.db_pool).await;

        match result {
            Ok(row) => {
                let token_long: &str = &row.tokens.unwrap();
                let mut tokens = token_long.split(',').collect::<Vec<&str>>();
                
                for i in 0..tokens.len() {
                    if tokens.get(i).unwrap().eq(&token.as_str()) {
                        tokens.remove(i);
                        
                        let mega_token = tokens.join(",").to_string();
                        let result = query!("UPDATE user SET tokens = ? WHERE username = ?", mega_token, username)
                            .execute(&self.db_pool).await;

                        return match result {
                            Ok(_) => Ok(()),
                            Err(_) => {
                                error!("Unable to update tokens on sign_out()! \
                                username -> {}, (previous) token_long -> {}, mega_token -> {}, token -> {}",
                                    username, token_long, mega_token, token);
                                Err(Error)
                            }
                        };
                    }
                }

                error!("Unauthorized request made for sign_out()! username -> {}, token -> {}", username, token);
                Err(Unauthorized)
            },
            Err(_) => {
                error!("Invalid username in request for get_avatar_for_user()! username -> {}", username);
                Err(InvalidUsername)
            },
        }
    }

    async fn get_avatar_for_user(self, _: Context, username: String) -> Result<String, ErrorCode> {
        info!("API Request: get_avatar_for_user( username -> {} )", username);
        
        let result = query!("SELECT avatar FROM user WHERE username = ?", username).fetch_one(&self.db_pool).await;

        match result {
            Ok(row) => Ok(row.avatar),
            Err(_) => {
                error!("Invalid username in request for get_avatar_for_user()! username -> {}", username);
                Err(InvalidUsername)
            }
        }
    }
}