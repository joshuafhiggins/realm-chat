use std::net::SocketAddr;

use sha3::{Digest, Sha3_256};
use sha3::digest::Update;
use sqlx::{MySql, Pool, Row};
use tarpc::context::Context;

use crate::types::{AuthUser, ErrorCode, RealmAuth};

#[derive(Clone)]
pub struct RealmAuthServer {
    pub socket: SocketAddr,
    pub db_pool: Pool<MySql>,
}

impl RealmAuthServer {
    pub fn new(socket: SocketAddr, db_pool: Pool<MySql>) -> RealmAuthServer {
        RealmAuthServer {
            socket,
            db_pool,
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

    async fn create_account(self, _: Context, username: String, email: String, avatar: String) -> Result<String, ErrorCode> {
        todo!()
    }

    async fn create_login_flow(self, _: Context, username: String) -> ErrorCode {
        todo!()
    }

    async fn create_token_from_login(self, _: Context, username: String, login_code: u16) -> Result<String, ErrorCode> {
        todo!()
    }

    async fn change_email_flow(self, _: Context, username: String, token: String) -> ErrorCode {
        todo!()
    }

    async fn resolve_email_flow(self, _: Context, username: String, token: String, login_code: u16, new_email: String) -> ErrorCode {
        todo!()
    }

    async fn change_username(self, _: Context, username: String, token: String, new_username: String) -> ErrorCode {
        todo!()
    }

    async fn change_avatar(self, _: Context, username: String, token: String, avatar: String) -> ErrorCode {
        todo!()
    }

    async fn get_all_data(self, _: Context, username: String, token: String) -> Result<AuthUser, ErrorCode> {
        todo!()
    }

    async fn get_avatar_for_user(self, _: Context, username: String) -> Result<String, ErrorCode> {
        todo!()
    }
}