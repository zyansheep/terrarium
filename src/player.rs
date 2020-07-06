#![allow(dead_code)]
#![allow(non_upper_case_globals)]

#![allow(unused_imports)]

use std::io;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use variant_encoding::{VarStringReader, VarStringWriter};

bitflags! {
	#[derive(Default)]
	struct Difficulty: u8 {
		const Softcore 		= 0b00000000; // 0
		const Mediumcore 	= 0b00000001; // 1
		const Harcore 		= 0b00000010; // 2
		const ExtraAccessory= 0b00000100; // 4
		const Creative 		= 0b00001000; // 8
	}
}
bitflags! {
	#[derive(Default)]
	struct TorchState: u8 {
		const UsingBiomeTorches = 0b00000001; // 1
		const HappyFunTorchTime = 0b00000010; // 2
	}
}
#[derive(Default, Debug)]
struct Color {
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
#[derive(Default, Debug)]
struct Inventory {

}

#[derive(Default, Debug)]
struct Appearance {
	skin: u8,
	hair: u8,
	hair_dye: u8,
	hide_visuals_1: u8,
	hide_visuals_2: u8,
	hide_misc: u8,
	hair_color: Color,
	skin_color: Color,
	eye_color: Color,
	shirt_color: Color,
	under_shift_color: Color,
	pants_color: Color,
	shoe_color: Color,
}

#[derive(Default, Debug)]
pub struct Player {
	pub id: u8,
	pub name: String,
	
	inventory: Inventory,
	appearance: Appearance,

	difficulty: Difficulty,
	torch_state: TorchState,
}

quick_error!{
	#[derive(Debug)]
	pub enum PlayerParseError {
		IO(err: io::Error){ from() }
		InvalidID(id_received: u8, id_have: u8) {
			display("Invalid ID Received: {}, have: {}", id_received, id_have)
		}
	}
}

use std::error::Error;
impl Player {
	pub fn parse_player_info_packet(&mut self, reader: &mut impl io::BufRead) -> Result<(), PlayerParseError> {
		let parsed_id = reader.read_u8()?;
		if self.id != parsed_id {
			return Err(PlayerParseError::InvalidID(parsed_id, self.id));
		}
		self.appearance.skin = reader.read_u8()?;
		self.appearance.hair = reader.read_u8()?;
		
		self.name = reader.read_varstring()?;
		
		self.appearance.hair_dye = reader.read_u8()?;
		self.appearance.hide_visuals_1 = reader.read_u8()?;
		self.appearance.hide_visuals_2 = reader.read_u8()?;
		self.appearance.hide_misc = reader.read_u8()?;
		self.appearance.hair_color = Color::read(reader)?;
		self.appearance.skin_color = Color::read(reader)?;
		self.appearance.eye_color = Color::read(reader)?;
		self.appearance.shirt_color = Color::read(reader)?;
		self.appearance.under_shift_color = Color::read(reader)?;
		self.appearance.pants_color = Color::read(reader)?;
		self.appearance.shoe_color = Color::read(reader)?;
		
		Ok(())
	}
}