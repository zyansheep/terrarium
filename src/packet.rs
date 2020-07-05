
#![allow(dead_code)]
use std::io;
use std::error::Error;
use byteorder::{ReadBytesExt, LittleEndian};
use variant_encoding::{VarStringReader, VarIntReader};

use crate::player::Player;

quick_error! {
	#[derive(Debug)]
	pub enum PacketError {
		UnknownPacket(pkt_type: u8) {
			display(x) -> ("Unknown Packet Type: {}", pkt_type)
		}
		IO(err: io::Error){ from() }
		
	}
}

//File that reads terraria's packets into nice little structures
#[derive(Debug)]
pub enum Packet {
	// Packets that are received only
	ConnectRequest(String), // Client asks server if correct version
	RequestWorldData(),
	
	// Packets that are sent out
	WorldInfo(), // Information about the world TODO: filter
	
	// Packets that are received, (possibly modified) and then broadcast to all clients
	PlayerInfo(Player),
}

impl Packet {
	pub fn read(reader: &mut impl io::BufRead) -> Result<Packet, PacketError> {
		// "Read" whole buffer (doesn't actually advance read cursor)
		println!("Buffer: {:?}", reader.fill_buf()?);
		
		reader.consume(1); // Ignore first 2 bytes?
		//reader.read_u16::<LittleEndian>()?;
		
		// Read packet type (Byteorder doesn't have read_u8 function?)
		let mut msg_type_slice = [10u8; 1];
		reader.read_exact(&mut msg_type_slice)?;
		reader.consume(1);
		let msg_type = msg_type_slice[0];
		
		println!("Recevied Packet Type: {}", msg_type);
		
		use Packet::*;
		match msg_type {
			0 => Ok( ConnectRequest(reader.read_varstring()?) ),
			_ => Err(PacketError::UnknownPacket(msg_type)),
		}
	}
}
