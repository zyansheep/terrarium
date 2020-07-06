
#![allow(dead_code)]
#![allow(unused_imports)]
use std::io;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use variant_encoding::{VarStringReader, VarIntReader, VarIntWriter};
use tokio_util::codec::{Decoder, Encoder};
use bytes::{BytesMut, BufMut, Bytes, Buf};

use crate::player::{Player, PlayerParseError };

pub mod types;
use types::NetworkText;

quick_error! {
	#[derive(Debug)]
	pub enum PacketError {
		IO(err: io::Error){ from() }
		UnknownType(pkt_type: u8) {
			display("Unknown Packet Type: {}", pkt_type)
		}
		UnimplementedPacket {
			display("Packet Not Implemented")
		}
		InvalidSize(packet_size: usize, buffer_size: usize) {
			display("Invalid Packet Size: Told: {}, Found: {}", packet_size, buffer_size)
		}
		
		// Specific Parsing Errors
		PlayerParse(err: PlayerParseError) { from() }
	}
}

//File that reads terraria's packets into nice little structures
#[derive(Debug)]
pub enum Packet {
	Empty(), // Default value
	
	// Packets that are received only
	ConnectRequest(String), // Client asks server if correct version
	WorldDataRequest(),
	
	// Packets that are sent out to individual clients
	SetUserSlot(u8), // Tell client what to refer to themselves as (why is this a single byte???)
	WorldInfo(), // Information about the world TODO: filter
	Disconnect(NetworkText),
	
	// Packets that are received, (possibly modified) and then broadcast to all clients
	PlayerInfo(Player),
}

#[derive(Default)]
pub struct PacketCodec;
impl Decoder for PacketCodec {
	type Item = Packet;
	type Error = PacketError;
	fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
		if src.remaining() == 0 { return Ok(None) } // No data to read
		println!("Started Reading Packet: Bytes: {:?}, Size: {:?}", src.bytes(), src.remaining());
		
		let mut reader = src.bytes();
		let size: usize = reader.read_varint()?;
		if size != src.remaining() { return Err(PacketError::InvalidSize(size, src.remaining())) } // Error: packet size doesn't match what packet says its size should be
		
		reader.read_u8()?;
		let msg_type = reader.read_u8()?;
		println!("Recevied Packet Type: {}", msg_type);
		
		let packet: Result<Option<Self::Item>, Self::Error>;
		use Packet::*;
		match msg_type {
			1 => packet = Ok(Some(ConnectRequest(reader.read_varstring()?))),
			4 => packet = {
				let mut player = Player::default();
				player.parse_player_info_packet(&mut reader)?;
				Ok(Some(PlayerInfo(player)))
			}, // Construct player struct
			_ => packet = Err( PacketError::UnknownType(msg_type) ),
		};
		src.advance(src.remaining()); // Treat packet as entirely read
		println!("Finished Reading Packet: Bytes: {:?}, Size: {:?}", src.bytes(), src.remaining());
		
		packet
	}
}
impl Encoder<&Packet> for PacketCodec {
	type Error = PacketError;
	fn encode(&mut self, item: &Packet, dst: &mut BytesMut) -> Result<(), Self::Error> {
		println!("Started Writing Packet: Bytes Left: {:?}", dst.remaining_mut());
		let mut writer = vec![];
		use Packet::*;
		match item {
			SetUserSlot(id) => {
				writer.write_u8(4)?;
				writer.write_u8(0)?;
				writer.write_u8(3)?; // Set User Slot Packet ID = 3
				writer.write_u8(*id)?; // Write ID
			},
			Disconnect(text) => {
				text.write(&mut writer)?;
			}
			_ => return Err(PacketError::UnimplementedPacket),
		};
		
		dst.put_slice(&writer[..]); // flush writer to destination
		println!("Finished Writing Packet: Bytes Left: {:?}, Data: {:?}", dst.remaining_mut(), dst.bytes());
		Ok(())
	}
}