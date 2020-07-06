
#![allow(dead_code)]
#![allow(unused_imports)]
use std::io;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use variant_encoding::{VarStringReader};
use tokio_util::codec::{Decoder, Encoder};
use bytes::{BytesMut, BufMut, Bytes, Buf};

use crate::player::Player;

quick_error! {
	#[derive(Debug)]
	pub enum PacketError {
		IO(err: io::Error){ from() }
		UnknownPacket(pkt_type: u8) {
			display("Unknown Packet Type: {}", pkt_type)
		}
		UnimplementedPacket {
			display("Packet Not Implemented")
		}
	}
}

//File that reads terraria's packets into nice little structures
#[derive(Debug)]
pub enum Packet {
	Empty(), // Default value
	// Packets that are received only
	ConnectRequest(String), // Client asks server if correct version
	RequestWorldData(),
	
	// Packets that are sent out to individual clients
	SetUserSlot(u8), // Tell client what to refer to themselves as (why is this a single byte???)
	WorldInfo(), // Information about the world TODO: filter
	
	// Packets that are received, (possibly modified) and then broadcast to all clients
	PlayerInfo(Player),
}

#[derive(Default)]
pub struct PacketCodec;
impl Decoder for PacketCodec {
	type Item = Packet;
	type Error = PacketError;
	fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
		if src.remaining() == 0 { return Ok(None) }
		println!("Started Reading Packet: Bytes: {:?}, Size: {:?}", src.bytes(), src.remaining());
		src.get_u8();
		let msg_type = src.get_u8();
		src.advance(1);
		println!("Bytes: {:?}, Size: {:?}", src.bytes(), src.remaining());
		println!("Recevied Packet Type: {}", msg_type);
		
		let mut reader = src.bytes();
		let packet;
		use Packet::*;
		match msg_type {
			0 => packet = ConnectRequest(reader.read_varstring()?),
			_ => return Err( PacketError::UnknownPacket(msg_type) ),
		};
		src.advance(src.remaining());
		println!("Finished Reading Packet: Bytes: {:?}, Size: {:?}", src.bytes(), src.remaining());

		//panic!();
		Ok(Some(packet))
	}
}
impl Encoder<&Packet> for PacketCodec {
	type Error = PacketError;
	fn encode(&mut self, item: &Packet, dst: &mut BytesMut) -> Result<(), Self::Error> {
		let writer = dst;
		use Packet::*;
		match *item {
			SetUserSlot(id) => writer.put_u8(id),
			_ => return Err(PacketError::UnimplementedPacket),
		};
		Ok(())
	}
}