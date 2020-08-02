#![allow(dead_code)]
#![allow(unused_imports)]

use log::{trace, error, warn, debug};

use std::{
	io::{Read, Write, BufReader, BufWriter},
	error::Error,
	sync::Arc,
	convert::TryInto,
	collections::HashMap
};
use rand::Rng;

use serde::{Deserialize, Serialize};
use flate2::{read::DeflateDecoder, write::DeflateEncoder, Compression};
use tokio::sync::{mpsc, Mutex};
use arc_swap::ArcSwap;

use variant_encoding::{VarStringWriter};
use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};

use crate::server::*;

pub mod chunk;
pub mod generator;
pub mod vanilla;
pub mod world_types;

pub use chunk::{Chunk, ChunkAction, ChunkThread, LoadedChunk, ChunkActionSender, ChunkCoord, TileCoord};
pub use generator::WorldGenerator;
pub use world_types::*;

use crate::server::{ClientActionSender, ServerActionSender};

#[derive(Debug)]
pub enum WorldAction {
	SpawnClient(ClientActionSender, Option<TileCoord>), // Send back chunk thread
	RequestWorldInfo(ClientActionSender), // Sends back cached world info
}
pub type WorldActionSender = mpsc::Sender<WorldAction>;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct World {
	pub name: String,
	
	pub id: i32, // Used for client side rendering
	pub uuid: u128, // Used for client side maps
	
	pub gen_data: GenerationData,
	pub dimensions: Dimensions,
	pub worldmode: WorldMode,
	pub style: Style,
	
	pub spawn_coord: TileCoord,
	pub dungeon_coord: TileCoord,
	
	pub progress: Progress,
	pub time: Time,
	pub weather: Weather,
	pub events: Events,
	
	pub angler: Angler,
	
	pub chunk_size: u16,
}

impl World {
	pub fn new(name: &str, seed: Option<&str>, gen: &mut impl WorldGenerator) -> Self {
		let mut world = World {
			name: name.into(), id: rand::random(), uuid: rand::random(),
			gen_data: gen.calculate_gen_data(),
			..World::default() // Default initialize the rest
		};
		
		use std::collections::hash_map::DefaultHasher;
		use std::hash::{Hasher, Hash};
		if let Some(seed) = seed {
			let mut hasher = DefaultHasher::new();
			seed.hash(&mut hasher);
			world.gen_data.seed = hasher.finish();
		}
		world
	}
	pub fn read(reader: &mut impl Read) -> Result<World, Box<dyn Error>> {
		trace!("Uncompressing World Data");
		let mut reader = DeflateDecoder::new(reader);
		
		trace!("Deserializing World Data");
		Ok(bincode::deserialize_from(&mut reader)?)
	}
	pub fn write(&self, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
		trace!("Compressing World Data");
		let mut writer = DeflateEncoder::new(writer, Compression::default());
		
		trace!("Serializing World Data");
		bincode::serialize_into(&mut writer, self)?;
		writer.finish()?;

		Ok(())
	}
	pub fn update_worldinfo_cache(&self, cache: ArcSwap<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		let mut writer = Vec::with_capacity(22 + 4 + self.name.len() + 170);
		
		writer.write_i32::<LittleEndian>(self.time.time as i32)?; // Time
		if self.events.is_blood_moon_happening { // Day and Moon Info (1 = Day Time, 2 = Blood Moon, 4 = Eclipse)
			writer.write_u8(2)?;
		} else if self.events.is_eclipse_happening {
			writer.write_u8(4)?;
		} else if self.time.is_day {
			writer.write_u8(1)?;
		} else { writer.write_u8(0)?; }

		writer.write_u8(self.time.moon_phase)?; // Moon Phase
		writer.write_i16::<LittleEndian>(self.dimensions.tile_width as i16)?; // Max Tiles X
		writer.write_i16::<LittleEndian>(self.dimensions.tile_height as i16)?; // Max Tiles Y
		writer.write_i16::<LittleEndian>(self.spawn_coord.x as i16)?; // Spawn X
		writer.write_i16::<LittleEndian>(self.spawn_coord.y as i16)?; // Spawn Y
		writer.write_i16::<LittleEndian>(self.gen_data.surface_y as i16)?;
		writer.write_i16::<LittleEndian>(self.gen_data.rock_layer_y as i16)?;
		writer.write_i32::<LittleEndian>(self.id)?;
		writer.write_varstring(&self.name)?;
		writer.write_u8(self.worldmode as u8)?;
		writer.write_u128::<LittleEndian>(self.uuid)?;
		writer.write_u64::<LittleEndian>(1)?; //idk what this is "self Generator Version" (seems to be 1 in packet analysis)
		writer.write_u8(self.style.moon_style)?; // ? Moon Type (set to 225 in packet analysis)
		
		writer.write_u8(self.style.forest_bg[0])?;
		writer.write_u8(self.style.corruption_bg)?;
		writer.write_u8(self.style.jungle_bg)?;
		writer.write_u8(self.style.snow_bg)?;
		writer.write_u8(self.style.hallow_bg)?;
		writer.write_u8(self.style.crimson_bg)?;
		writer.write_u8(self.style.desert_bg)?;
		writer.write_u8(self.style.ocean_bg)?;
		writer.write_u8(self.style.mushroom_bg)?;
		writer.write_u8(self.style.underworld_bg)?;
		writer.write(&self.style.forest_bg[1..=3])?;
		
		writer.write_u8(self.style.snow_bg_style)?;
		writer.write_u8(self.style.jungle_bg_style)?;
		writer.write_u8(self.style.underworld_bg_style)?;
		
		writer.write_f32::<LittleEndian>(self.weather.wind_speed)?;
		writer.write_u8(self.weather.num_clouds as u8)?;
		for i in self.style.forest_bg_x.iter() {
			writer.write_i32::<LittleEndian>(*i as i32)?;
		}
		writer.write(&self.style.forest_bg_style[..])?; // Tree Style 1-4
		
		for i in self.style.cave_bg_x.iter() {
			writer.write_i32::<LittleEndian>(*i as i32)?;
		}
		
		writer.write(&self.style.cave_bg_style[..])?;
		writer.write(&self.style.tree_tops[..])?;
		
		writer.write_f32::<LittleEndian>(self.weather.rain_amount)?;
		
		writer.write(&[0u8; 7][..])?; // Skipping event info
		
		writer.write_i16::<LittleEndian>(self.gen_data.copper_tier as i16)?;
		writer.write_i16::<LittleEndian>(self.gen_data.iron_tier as i16)?;
		writer.write_i16::<LittleEndian>(self.gen_data.silver_tier as i16)?;
		writer.write_i16::<LittleEndian>(self.gen_data.gold_tier as i16)?;
		writer.write_i16::<LittleEndian>(self.gen_data.cobalt_tier as i16)?;
		writer.write_i16::<LittleEndian>(self.gen_data.mythril_tier as i16)?;
		writer.write_i16::<LittleEndian>(self.gen_data.adamantite_tier as i16)?;
		
		writer.write_i8(self.events.invasion_type as i8)?; // Invasion Type (SByte)
		writer.write_u64::<LittleEndian>(0)?; //Lobby ID
		writer.write_f32::<LittleEndian>(self.weather.sandstorm_severity)?;
		
		cache.store(Arc::new(writer));
		Ok(())
	}
	pub async fn handle(&mut self, mut action_receiver: mpsc::Receiver<WorldAction>) -> Result<(), Box<dyn Error>> {
		let mut chunks: HashMap<ChunkCoord, Option<LoadedChunk>> = HashMap::new();
		
		let spawn_chunk_coord = ChunkCoord::from_tilecoord(&self.spawn_coord, self.chunk_size);
		chunks.insert(spawn_chunk_coord, Some(LoadedChunk::from_chunk(
			Chunk::test_chunk(self.chunk_size)
		)));
		let world_info: ArcSwap<Vec<u8>> = ArcSwap::new(Arc::new(Vec::new())); // Initialize World ArcSwap Cache
		
		loop {
			if let Some(action) = action_receiver.recv().await {
				use WorldAction::*;
				match action {
					// From Clients
					SpawnClient(mut sender, tile_coord) => {
						// parse chunk coord for sent tile coord or self.spawn_coord if tile coord not specified
						let spawn_coord = tile_coord.unwrap_or(self.spawn_coord);
						let spawn_chunk_coord = ChunkCoord::from_tilecoord(&spawn_coord, self.chunk_size);
						if let Some(spawn_chunk) = chunks.get_mut(&spawn_chunk_coord) {
							if let Some(loaded_chunk) = spawn_chunk {
								let result = loaded_chunk.get_chunk_handle().await;
								match result {
									Ok(handler) => {
										let h_cp = handler.clone();
										sender.send(ClientAction::UpdateChunkHandler(h_cp)).await?;
									},
									Err(err) => { error!("Failed to get/create thread sender for chunk at {:?} err: {:?}", spawn_chunk_coord, err); continue; },
								}
							} else {
								// TODO: Load chunk if not loaded and send asyncronously
								error!("Chunk is not loaded at {:?}", spawn_chunk_coord); 
							}
						} else { warn!("Client attempted to load chunk outside of world"); }
					}
					RequestWorldInfo(mut sender) => {
						sender.send(ClientAction::SendPacket(packet::Packet::WorldInfo(world_info.clone()))).await?;
					},
					// From Chunks
					
				}
			} else { return Ok(()) }
		}
	}
}
