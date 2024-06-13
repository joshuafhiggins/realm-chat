#[tarpc::service]
pub trait RealmChat {
	async fn test(name: String) -> String;
}