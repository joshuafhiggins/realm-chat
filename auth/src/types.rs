
#[tarpc::service]
pub trait RealmAuth {
    async fn test(name: String) -> String;
}