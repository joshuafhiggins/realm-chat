use durian::bincode_packet;

#[bincode_packet]
pub struct Greet {
	id: u32
}