use serde::{Deserialize, Serialize};

#[tarpc::service]
pub trait RealmAuth {
    async fn test(name: String) -> String;
    async fn server_token_validation(username: String, server_id: String, domain: String, tarpc_port: u16) -> bool;
    async fn create_account(username: String, email: String, avatar: String) -> Result<String, ErrorCode>;
    async fn create_login_flow(username: String) -> ErrorCode;
    async fn create_token_from_login(username: String, login_code: u16) -> Result<String, ErrorCode>;
    
    //NOTE: Need to be the user
    async fn change_email_flow(token: String) -> ErrorCode;
    async fn resolve_email_flow(token: String, login_code: u16, new_email: String) -> ErrorCode;
    async fn change_username(token: String, new_username: String) -> ErrorCode;
    async fn change_avatar(token: String, avatar: String) -> ErrorCode;
    //TODO:
    // Create account
    // Change email
    // Change username
    // Change/Upload/Delete avatar
    // OAuth login, check against email, store token, take avatar
    //      Google, Apple, GitHub, Discord
    // Get avatar
    // Get all userdata if you are the user
    // Server token validation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCode {
    None,
    Error,
    EmailTaken,
    UsernameTaken,
    InvalidLoginCode,
}