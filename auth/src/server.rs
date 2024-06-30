use std::net::SocketAddr;
use tarpc::context::Context;
use crate::types::RealmAuth;

#[derive(Clone)]
pub struct RealmAuthServer {
    pub socket: SocketAddr,
}

impl RealmAuthServer {
    pub fn new(socket: SocketAddr) -> RealmAuthServer {
        RealmAuthServer {
            socket,
        }
    }
}

impl RealmAuth for RealmAuthServer {
    async fn test(self, context: Context, name: String) -> String {
        format!("Hello {}", name)
    }
}