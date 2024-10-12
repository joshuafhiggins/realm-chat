use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorCode {
    Error,
    Unauthorized,
    EmailTaken,
    UsernameTaken,
    InvalidLoginCode,
    InvalidImage,
    InvalidUsername,
    InvalidEmail,
    InvalidToken,
    UnableToConnectToMail,
    UnableToSendMail,
    AlreadyJoinedServer,
    NotInServer,
    
    MessageNotFound,
    RoomNotFound,
    UserNotFound,
    DepthTooLarge,
    MalformedDBResponse,
    
    RPCError,
}