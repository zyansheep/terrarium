
#![allow(dead_code)]
#![allow(unused_imports)]
use std::io;
use std::sync::Arc;
use tokio::sync::RwLockReadGuard;
use tokio_util::codec::{Decoder, Encoder};
use bytes::{BytesMut, BufMut, Bytes, Buf, buf::{BufExt, BufMutExt}};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use variant_encoding::{VarStringReader, VarIntReader, VarIntWriter};

use crate::player::{self, Player, PlayerError};
use crate::server::cache::{WorldInfo};

pub mod types;
use types::*;

#[derive(Error, Debug)]
pub enum PacketError {
	#[error("Error parsing packet")]
	CodecError(#[from] io::Error),
	#[error("Invalid Packet Data")]
	InvalidField,
	#[error("Unknown Packet Type: {0}")]
	UnknownType(u8),
	#[error("Packet Not Implemented")]
	Unimplemented,
	#[error("Invalid Packet Size: Told: {told}, Found: {found}")]
	InvalidSize{told: usize, found: usize}
}

//File that reads terraria's packets into nice little structures
#[derive(Debug)]
pub enum Packet {
	Empty(), // Default value
	
	// Packets that are received only
	ConnectRequest(String), // Client asks server if correct version
	WorldDataRequest,
	EssentialTilesRequest(i32, i32),
	PlayerUUID(String),
	
	// Packets that are sent out to individual clients
	SetUserSlot(u8), // Tell client what to refer to themselves as (why is this a single byte???)
	WorldInfo(WorldInfo), // Information about the world TODO: filter
	Disconnect(NetworkText),
	Status(i32, NetworkText, u8),
	
	// Packets that are received, (possibly modified) and then broadcast to all clients
	PlayerAppearance(player::Appearance),
	PlayerHp{hp: u16, max_hp: u16},
	PlayerMana{mana: u16, max_mana: u16},
	PlayerBuff{buffs: [u16; 22]},
	PlayerInventorySlot{slot_index: u16, amount: u16, item_prefix: u8, net_id: u16},
}

#[derive(Default)]
pub struct PacketCodec;
impl Decoder for PacketCodec {
	type Item = Packet;
	type Error = PacketError;
	fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
		if src.remaining() == 0 { return Ok(None) } // No data to read
		//println!("Started Reading Packet: Bytes: {:?}, Size: {:?}", src.bytes(), src.remaining());
		
		let bytes_left = src.remaining();
		let size = src.get_u16_le() as usize; //First 2 bytes are size (sometimes?) TODO: figure this out
		// NetMessage.cs only errors when bytes_left is less than read size
		if size > bytes_left { return Err(PacketError::InvalidSize{told: size, found: bytes_left}) } // Error: packet size doesn't match what packet says its size should be
		
		log::debug!("Read Bytes: {:02x?}", &src.bytes()[0..size-2]);
		
		//println!("Bytes Left: {}", src.remaining());
		let mut reader = src.reader();
		
		let msg_type = reader.read_u8()?;
		//println!("Recevied Packet Type: {}", msg_type);

		use Packet::*;
		let packet = match msg_type {
			1 => ConnectRequest(reader.read_varstring()?),
			4 => {
				reader.read_u8()?; // Read Player ID
				PlayerAppearance(player::Appearance::read(&mut reader)?)
			}, // Construct player struct
			68 => PlayerUUID(reader.read_varstring()?),
			16 => { // Player Health
				reader.read_u8()?; // Read Player ID
				PlayerHp {
					hp: reader.read_u16::<LittleEndian>()?,
					max_hp: reader.read_u16::<LittleEndian>()?
				}
			},
			42 => { // Player Mana
				reader.read_u8()?; // Read Player ID
				PlayerMana {
					mana: reader.read_u16::<LittleEndian>()?,
					max_mana: reader.read_u16::<LittleEndian>()?
				}
			}
			50 => { // Player Buffs
				reader.read_u8()?; // Read Player ID
				let mut buffs = [0u16; 22];
				reader.read_u16_into::<LittleEndian>(&mut buffs)?;
				PlayerBuff {
					buffs: buffs
				}
			}
			5 => {
				reader.read_u8()?; // Read Player ID
				PlayerInventorySlot {
					slot_index: reader.read_u16::<LittleEndian>()?,
					amount: reader.read_u16::<LittleEndian>()?,
					item_prefix: reader.read_u8()?,
					net_id: reader.read_u16::<LittleEndian>()?,
				}
			}
			6 => WorldDataRequest,
			8 => EssentialTilesRequest(
				reader.read_i32::<LittleEndian>()?,
				reader.read_i32::<LittleEndian>()?,
			),
			_ => Packet::Empty(),
		};
		//println!("Finished Reading Packet: Bytes: {:?}, Size: {:?}", src.bytes(), src.remaining());
		//println!("Finished Reading Packet: Bytes: {:?}, Size: {:?}", src.bytes(), src.remaining());
		
		if std::mem::discriminant(&packet) == std::mem::discriminant(&Packet::Empty()) { return Err(PacketError::UnknownType(msg_type)) }
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
				writer.write_u16::<LittleEndian>(4)?;
				writer.write_u8(3)?; // Set User Slot Packet ID = 3
				writer.write_u8(*id)?; // Write ID
			},
			Disconnect(localized_text) => {
				localized_text.write(&mut writer)?;
				dst.put_u16_le(writer.len() as u16 + 2);
			}
			WorldInfo(info) => { // Receives locked reader (managed by calling function)
				writer.write_u16::<LittleEndian>(info.data.len() as u16 + 3)?;
				writer.write_u8(7)?; // Packet ID
				use std::io::Write;
				writer.write(&info.data[..])?;
				//println!("Data Length: {}", writer.len());
				//println!("Data Hex: {:02X?}", &writer[..]);
			}
			Status(max, localized_text, flags) => {
				writer.write_i32::<LittleEndian>(*max)?;
				localized_text.write(&mut writer)?;
				writer.write_u8(*flags)?;
				
				dst.put_u16_le(writer.len() as u16 + 2);
			}
			_ => return Err(PacketError::Unimplemented),
		};
		
		dst.put_slice(&writer[..]); // flush writer to destination
		log::debug!("Write Bytes: {:02x?}", dst.bytes());
		Ok(())
	}
}