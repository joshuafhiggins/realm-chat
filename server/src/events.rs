// use durian::bincode_packet;
// use crate::types::{Message, Room, User};
// 
// #[bincode_packet]
// pub struct Greet {
// 	pub id: u32
// }
// 
// #[bincode_packet]
// pub struct UserJoinedEvent {
// 	pub user: User,
// }
// 
// #[bincode_packet]
// pub struct UserLeftEvent {
// 	pub user: User,
// }
// 
// #[bincode_packet]
// pub struct NewMessageEvent {
// 	pub message: Message,
// }
// 
// // #[bincode_packet]
// // pub struct StartTypingEvent {
// // 	pub user: User,
// // 	pub room: String,
// // }
// // 
// // #[bincode_packet]
// // pub struct StopTypingEvent {
// // 	pub user: User,
// // 	pub room: String,
// // }
// 
// #[bincode_packet]
// pub struct NewRoomEvent {
// 	pub room: Room,
// }
// 
// #[bincode_packet]
// pub struct DeleteRoomEvent {
// 	pub roomid: String,
// }
// 
// #[bincode_packet]
// pub struct KickedUserEvent {
// 	pub userid: String, 
// }
// 
// #[bincode_packet]
// pub struct BannedUserEvent {
// 	pub userid: String,
// }
// 
// #[bincode_packet]
// pub struct PromotedUserEvent {
// 	pub userid: String,
// }
// 
// #[bincode_packet]
// pub struct DemotedUserEvent {
// 	pub userid: String,
// }