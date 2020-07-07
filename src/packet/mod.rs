
#![allow(dead_code)]
#![allow(unused_imports)]
use std::io;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use variant_encoding::{VarStringReader, VarIntReader, VarIntWriter};
use tokio_util::codec::{Decoder, Encoder};
use bytes::{BytesMut, BufMut, Bytes, Buf};

use crate::player::{self, Player, PlayerError};

pub mod types;
use types::NetworkText;

#[derive(Error, Debug)]
pub enum PacketError {
	#[error("Error parsing packet")]
	CodecError(#[from] io::Error),
	#[error("Unknown Packet Type: {0}")]
	UnknownType(u8),
	#[error("Packet Not Implemented")]
	Unimplemented,
	#[error("Invalid Packet Size: Told: {0}, Found: {1}")]
	InvalidSize(usize, usize)
}

//File that reads terraria's packets into nice little structures
#[derive(Debug, PartialEq)]
pub enum Packet {
	Empty(), // Default value
	
	// Packets that are received only
	ConnectRequest(String), // Client asks server if correct version
	WorldDataRequest(),
	PlayerUUID(String),
	
	// Packets that are sent out to individual clients
	SetUserSlot(u8), // Tell client what to refer to themselves as (why is this a single byte???)
	WorldInfo(), // Information about the world TODO: filter
	Disconnect(NetworkText),
	
	// Packets that are received, (possibly modified) and then broadcast to all clients
	PlayerAppearance(player::Appearance),
	PlayerHp{hp: u16, max_hp: u16},
	PlayerMana{mana: u16, max_mana: u16},
	PlayerBuff{buffs: [u16; 22]},
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
		let size = reader.read_u16::<LittleEndian>()? as usize; //First 2 bytes are size (sometimes?) TODO: figure this out
		let bytes_left = src.remaining();
		// NetMessage.cs only errors when bytes_left is less than read size
		if size > bytes_left { return Err(PacketError::InvalidSize(size, bytes_left)) } // Error: packet size doesn't match what packet says its size should be
		
		let msg_type = reader.read_u8()?;
		println!("Recevied Packet Type: {}", msg_type);
		

		use Packet::*;
		let packet = match msg_type {
			1 => ConnectRequest(reader.read_varstring()?),
			4 => {
				reader.read_u8()?; // Read Player ID
				PlayerAppearance(player::Appearance::read(&mut reader)?)
			}, // Construct player struct
			68 => PlayerUUID(reader.read_varstring()?),
			16 => {
				reader.read_u8()?; // Read Player ID
				PlayerHp {
					hp: reader.read_u16::<LittleEndian>()?,
					max_hp: reader.read_u16::<LittleEndian>()?
				}
			},
			42 => {
				reader.read_u8()?; // Read Player ID
				PlayerMana {
					mana: reader.read_u16::<LittleEndian>()?,
					max_mana: reader.read_u16::<LittleEndian>()?
				}
			}
			_ => Packet::Empty(),
		};
		println!("Finished Reading Packet: Bytes: {:?}, Size: {:?}", src.bytes(), src.remaining());
		src.advance(src.remaining()); // Treat packet as entirely read
		//println!("Finished Reading Packet: Bytes: {:?}, Size: {:?}", src.bytes(), src.remaining());
		
		if packet == Packet::Empty() { return Err(PacketError::UnknownType(msg_type)) }
		Ok(Some(packet))
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
				dst.put_u8(writer.len() as u8);
				dst.put_u8(0);
			}
			_ => return Err(PacketError::Unimplemented),
		};
		
		dst.put_slice(&writer[..]); // flush writer to destination
		println!("Finished Writing Packet: Bytes Left: {:?}, Data: {:?}", dst.remaining_mut(), dst.bytes());
		Ok(())
	}
}