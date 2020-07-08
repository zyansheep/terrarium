use std::io;
use std::convert::TryInto;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use variant_encoding::{VarStringReader, VarStringWriter};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
enum NetworkTextMode {
	Literal = 0u8, Formattable = 1, LocalizationKey = 2,
}
#[derive(Debug, PartialEq)]
pub struct NetworkText {
	mode: NetworkTextMode,
	text: String,
	substitution: Vec<NetworkText>,
}
impl NetworkText {
	pub fn new(string: &str) -> Self {
		NetworkText {
			mode: NetworkTextMode::Literal,
			text: string.to_owned(),
			substitution: vec![],
		}
	}
	pub fn write(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
		writer.write_u8(self.mode as u8)?;
		writer.write_varstring(&self.text)?;
		if self.mode != NetworkTextMode::Literal {
			let num: u8 = self.substitution.len().try_into().unwrap_or(255); //Impossible for this to get too large
			writer.write_u8(num)?;
			for text in self.substitution.iter() {
				text.write(writer)?;
			}
		}
		Ok(())
	}
}
#[derive(Debug, PartialEq)]
pub struct WorldInfo {
	time: i32,
	moon_info: u8,
	moon_phase: u8,
	max_tiles_x: i16,
	max_tiles_y: i16,
	spawn_x: i16,
	spawn_y: i16,
	
	world_surface: i16,
	rock_layer: i16,
	
	world_id: i32,
	world_name: String,
	game_mode: u8,
	world_unique_id: [u8; 16],
	world_generator_version: u64,
	
	moon_style: u8,

	/*forest_bg_style: [u8; 4],

	cave_bg_x: [u32; 3],
	cave_bg_style: [u8; 4],*/
	
	forest_bg: u8,
	corruption_bg: u8,
	jungle_bg: u8,
	snow_bg: u8,
	hallow_bg: u8,
	crimson_bg: u8,
	desert_bg: u8,
	ocean_bg: u8,
	mushroom_bg: u8,
	underworld_bg: u8,
	other_tree_bg: [u8; 3], // idk what these are?
	
	snow_bg_style: u8,
	jungle_bg_style: u8,
	underworld_bg_style: u8,

	tree_tops: [u8; 13],
}