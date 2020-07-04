#![allow(dead_code)]

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use std::fs::File;
use std::io;
use std::error::Error;

use variant_encoding::{VarStringReader, VarStringWriter};

static VERSION: i32 = 230; // 1.4.0.5 version.

#[derive(Debug, Default)]
struct WorldState {
	downed_bosses: [bool; 9], // downedBoss 1, 2, 3, queenBee, mechboss 1, 2, 3, mechbossany, plantboss, golemboss, fishron, cultist, moonlord, halloween king, halloween tree, christmas queen, christmas santank, christmas tree.
	downed_events: [bool; 8], // Array of bools corresponding to various downed bosses.
	saved_npcs: [bool; 8],
	world_events: [bool; 8],
	player_cooldowns: [bool; 8],
	invasion_data: [bool; 8],
}

#[derive(Debug, Default)]
pub struct World {
	revision: u32,
	is_favorite: bool,
	tile_frame_important: Vec<bool>,
	name: String,
	seed: String,

	id: i32,
	unique_id: u128,
	world_gen_version: u64,
	left_world: i32,
	right_world: i32,
	top_world: i32,
	bottom_world: i32,
	max_tiles_y: i32,
	max_tiles_x: i32,
	world_state: WorldState,
}

impl World {
	pub fn read_from_file(path: &str) -> Result<World, Box<dyn Error>> {
		let file = File::open(path)?;
		let mut reader = io::BufReader::new(file);
		let wld = World::read(&mut reader)?;

		Ok(wld)
	}

	pub fn write_to_file(&self, path: &str) -> Result<(), Box<dyn Error>> {
		let file = File::create(path)?;
		let mut writer = io::BufWriter::new(file);
		self.write(&mut writer)?;

		Ok(())
	}

	pub fn read(reader: &mut impl io::BufRead) -> Result<World, Box<dyn Error>> {
		let mut wld = World::default();

		wld.read_file_format_header(reader)?;
		wld.read_world_header(reader)?;

		Ok(wld)
	}

	fn read_file_format_header(&mut self, reader: &mut impl io::BufRead) -> Result<(), Box<dyn Error>> {
		let file_version = reader.read_i32::<LittleEndian>()?; // Version (we are assuming that it is 230.)
		assert_eq!(file_version, VERSION, "Outdated world file");

		// File metadata.
		reader.read_u64::<LittleEndian>()?; // Magic + filetype (we are assuming that it is a world file.)
		self.revision = reader.read_u32::<LittleEndian>()?;
		self.is_favorite = reader.read_u64::<LittleEndian>()? != 0;

		// Chunk offsets.
		let chunk_count = reader.read_i16::<LittleEndian>()?;
		let mut chunk_offsets = vec![0; chunk_count as usize];
		for i in 0..chunk_count {
			chunk_offsets[i as usize] = reader.read_i32::<LittleEndian>()?;
		}

		// Tile frame important.
		let tile_count = reader.read_i16::<LittleEndian>()?;
		self.tile_frame_important = vec!(false; tile_count as usize);

		let mut byte = reader.read_u8()?;
		let mut bit = 0;
		for i in 0..tile_count {
			if byte & (1 << bit) != 0 {
				self.tile_frame_important[i as usize] = true;
			}
			bit += 1;
			if bit == 8 {
				byte = reader.read_u8()?;
				bit = 0;
			}
		}

		Ok(())
	}

	fn read_world_header(&mut self, reader: &mut impl io::BufRead) -> Result<(), Box<dyn Error>> {
		self.name = reader.read_varstring()?;
		self.seed = reader.read_varstring()?; // if VERSION >= 179
		self.world_gen_version = reader.read_u64::<LittleEndian>()?;
		self.unique_id = reader.read_u128::<LittleEndian>()?;

		self.id = reader.read_i32::<LittleEndian>()?;
		self.left_world = reader.read_i32::<LittleEndian>()?;
		self.right_world = reader.read_i32::<LittleEndian>()?;
		self.top_world = reader.read_i32::<LittleEndian>()?;
		self.bottom_world = reader.read_i32::<LittleEndian>()?;
		self.max_tiles_y = reader.read_i32::<LittleEndian>()?;
		self.max_tiles_x = reader.read_i32::<LittleEndian>()?;

		Ok(())
	}

	pub fn write(&self, writer: &mut (impl io::Write + io::Seek)) -> Result<(), Box<dyn Error>> {
		self.write_file_format_header(writer)?;
		self.write_world_header(writer)?;

		Ok(())
	}

	fn write_file_format_header(&self, writer: &mut (impl io::Write + io::Seek)) -> Result<(), Box<dyn Error>> {
		writer.write_i32::<LittleEndian>(230)?; // World File Version

		// File metadata
		writer.write(b"relogic")?; // Magic letters
		writer.write_u8(2)?; // Filetype
		writer.write_u32::<LittleEndian>(self.revision)?;
		writer.write_u64::<LittleEndian>(self.is_favorite as u64)?;

		// Chunk offsets
		writer.write_i16::<LittleEndian>(11)?; // Chunk count
		for _ in 0..11 {
			writer.write_i32::<LittleEndian>(0)?; // Placeholders for after writing all chunks
		}

		// Tile frame important
		writer.write_i16::<LittleEndian>(self.tile_frame_important.len() as i16)?;

		let mut byte = 0;
		let mut bit = 0;
		for i in 0..self.tile_frame_important.len() {
			if self.tile_frame_important[i] {
				byte |= 1 << bit;
			}
			bit += 1;
			if bit == 8 {
				writer.write_u8(byte)?;
				byte = 0;
				bit = 0;
			}
		}
		if bit != 0 {
			writer.write_u8(byte)?;
		}

		Ok(())
	}

	fn write_world_header(&self, writer: &mut (impl io::Write + io::Seek)) -> Result<(), Box<dyn Error>> {
		writer.write_varstring(&self.name)?;

		Ok(())
	}
}
