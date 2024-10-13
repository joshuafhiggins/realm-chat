use sha3::digest::Update;
use sha3::{Digest, Sha3_256};

pub mod types;

pub fn stoken(token: &str, serverid: &str, domain: &str, port: u16) -> String {
	let hash = Sha3_256::new().chain(format!("{}{}{}{}", token, serverid, domain, port)).finalize();
	hex::encode(hash)
}