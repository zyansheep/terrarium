#![allow(dead_code)]
#![allow(non_upper_case_globals)]

#![allow(unused_imports)]

use std::io;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use variant_encoding::{VarStringReader, VarStringWriter};

#[derive(Error, Debug)]
pub enum PlayerError {
	#[error("Error Reading / Writing Packet Data")]
	ReadError(#[from] io::Error),
	#[error("Packet received that wrote to unmodifiable field")]
	WrongField(&'static str),
}

bitflags! {
	#[derive(Default)]
	pub struct Difficulty: u8 {
		const Softcore 		= 0b00000000; // 0
		const Mediumcore 	= 0b00000001; // 1
		const Harcore 		= 0b00000010; // 2
		const ExtraAccessory= 0b00000100; // 4
		const Creative 		= 0b00001000; // 8
	}
}
bitflags! {
	#[derive(Default)]
	pub struct TorchState: u8 {
		const UsingBiomeTorches = 0b00000001; // 1
		const HappyFunTorchTime = 0b00000010; // 2
	}
}
#[derive(Default, Debug, PartialEq)]
pub struct Color {
	r: u8, g: u8, b: u8,
}
impl Color {
	pub fn read(reader: &mut impl io::BufRead) -> Result<Color, io::Error> {
		Ok(Color {
			r: reader.read_u8()?,
			g: reader.read_u8()?,
			b: reader.read_u8()?,
		})
	}
}
#[derive(Default, Debug, PartialEq)]
pub struct Inventory {

}

#[derive(Default, Debug, PartialEq)]
pub struct Appearance {
	pub name: String,
	
	pub skin: u8,
	pub hair: u8,
	pub hair_dye: u8,
	pub hide_visuals_1: u8,
	pub hide_visuals_2: u8,
	pub hide_misc: u8,
	pub hair_color: Color,
	pub skin_color: Color,
	pub eye_color: Color,
	pub shirt_color: Color,
	pub under_shift_color: Color,
	pub pants_color: Color,
	pub shoe_color: Color,
	
	pub unknown_color: Color,
	pub unknown_trait: u8,
	pub unknown_trait2: u8,
}
impl Appearance {
	pub fn read(reader: &mut impl io::BufRead) -> Result<Appearance, crate::packet::PacketError> {
		let mut appearance = Appearance::default();
		appearance.skin = reader.read_u8()?;
		appearance.hair = reader.read_u8()?;
		
		appearance.name = reader.read_varstring()?;
		
		appearance.hair_dye = reader.read_u8()?;
		appearance.hide_visuals_1 = reader.read_u8()?;
		appearance.hide_visuals_2 = reader.read_u8()?;
		appearance.hide_misc = reader.read_u8()?;
		appearance.hair_color = Color::read(reader)?;
		appearance.skin_color = Color::read(reader)?;
		appearance.eye_color = Color::read(reader)?;
		appearance.shirt_color = Color::read(reader)?;
		appearance.under_shift_color = Color::read(reader)?;
		appearance.pants_color = Color::read(reader)?;
		appearance.shoe_color = Color::read(reader)?;
		
		appearance.unknown_color = Color::read(reader)?;
		appearance.unknown_trait = reader.read_u8()?;
		appearance.unknown_trait2 = reader.read_u8()?;
		
		Ok(appearance)
	}
	pub fn init(&mut self, other: Appearance) -> Result<(), PlayerError> { 
		if self.name.is_empty() {
			Err(PlayerError::WrongField("Can't Modify Player Appearance"))
		} else {
			*self = other;
			println!("{:?}", self);
			Ok(())
		}
	}
}

#[derive(Default, Debug, PartialEq)]
pub struct Status {
	pub hp: u16,
	pub max_hp: u16,
	
	pub mana: u16,
	pub max_mana: u16,
	
	pub buffs: [u16; 22],
}
use crate::packet::Packet;
impl Status {
	pub fn init(&mut self, packet: Packet) -> Result<(), PlayerError> {
		match packet {
			Packet::PlayerHp{hp: _, max_hp} => {
				if self.max_hp == 0 {
					self.max_hp = max_hp;
					self.hp = self.max_hp;
					Ok(())
				} else { Err(PlayerError::WrongField("Can't Modify Hp")) }
			},
			Packet::PlayerMana{mana: _, max_mana} => {
				if self.max_mana == 0 {
					self.max_mana = max_mana;
					self.mana = self.max_mana;
					Ok(())
				} else { Err(PlayerError::WrongField("Can't Modify Mana")) }
			},
			Packet::PlayerBuff{buffs} => {
				if self.buffs == [0u16; 22] {
					self.buffs = buffs;
					Ok(())
				} else { Err(PlayerError::WrongField("Can't Modify Buffs")) }
			},
			_ => Err(PlayerError::WrongField("Unknown Status Packet")),
		}
	}
}

#[derive(Default, Debug, PartialEq)]
pub struct Player {
	pub id: u8,
	pub uuid: String, // uuid of the player TODO: what is this used for?
	
	pub status: Status, // Holds hp, mana, buffs etc.
	pub inventory: Inventory, // Whats in your inventory?
	pub appearance: Appearance, // Appearance information

	pub difficulty: Difficulty,
	pub torch_state: TorchState,
}