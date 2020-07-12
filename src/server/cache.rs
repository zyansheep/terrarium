
use log::{info, error};
use std::{error::Error, sync::Arc, io::Write};
use tokio::sync::{RwLock, mpsc};

use byteorder::{LittleEndian, WriteBytesExt, ByteOrder};
use variant_encoding::VarStringWriter;

use crate::world::World;
use crate::server;

/*
#[derive(Debug)]
pub enum CacheCommand {
	UpdateWorldInfo,
	UpdateChunk{x: u16, y: u16},
	RequestWorldInfo(server::ClientActionSender),
	RequestChunk(server::ClientActionSender),
}
#[derive(Default, Debug)]
pub struct Cache {
	pub world_info: Arc<RwLock<WorldInfo>>,
}
impl Cache {
	pub fn new(world: Arc<World>) -> (Cache, mpsc::Sender<CacheCommand>) {
		let mut cache = Cache::default();
		let (update, update_receiver) = mpsc::channel(100);
		
		(cache, update)
	}
	async fn start_thread(&mut self, world: Arc<World>, mut receiver: mpsc::Receiver<CacheCommand>) -> Result<(), Box<dyn Error>> {
		use CacheCommand::*;
		loop {
			if let Some(command) = receiver.recv().await {
				match command {
					UpdateWorldInfo => {
						info!("Updating World Info...");
						let worldinfo = WorldInfo::new(world.clone())?; // Parse new world info
						let mut lock = self.world_info.write().await; // Wait to aquire Write lock (this will go ahead of any new readers looking to read)
						*lock = worldinfo; // Replace object
					},
					RequestWorldInfo(mut client_action) => {
						info!("Received World Info Request");
						let action = Arc::new( server::ClientAction::SendWorldInfo(self.world_info.clone()) );
						client_action.send(action).await?;
					}
					_ => info!("Unimplemented Update Command: {:?}", command),
				}
			}
		}
	}
}*/

//Server Cache object, stores cached packets
#[derive(Default, Debug, Clone)]
pub struct WorldInfo {
	pub data: Vec<u8>,
}
impl WorldInfo {
	pub fn new(world: Arc<World>) -> Result<WorldInfo, Box<dyn Error>> {
		let mut info = WorldInfo::default();
		info.data = Vec::with_capacity(22 + 4 + world.name.len() + 170);
		let mut writer = info.data;
		
		writer.write_i32::<LittleEndian>(world.time.time as i32)?; // Time
		if world.events.is_blood_moon_happening { // Day and Moon Info (1 = Day Time, 2 = Blood Moon, 4 = Eclipse)
			writer.write_u8(2)?;
		} else if world.events.is_eclipse_happening {
			writer.write_u8(4)?;
		} else if world.time.is_day {
			writer.write_u8(1)?;
		} else { writer.write_u8(0)?; }

		writer.write_u8(world.time.moon_phase)?; // Moon Phase
		writer.write_i16::<LittleEndian>(world.dimensions.tile_width as i16)?; // Max Tiles X
		writer.write_i16::<LittleEndian>(world.dimensions.tile_height as i16)?; // Max Tiles Y
		writer.write_i16::<LittleEndian>(world.spawn_x as i16)?; // Spawn X
		writer.write_i16::<LittleEndian>(world.spawn_y as i16)?; // Spawn Y
		writer.write_i16::<LittleEndian>(world.surface_y as i16)?;
		writer.write_i16::<LittleEndian>(world.rock_layer_y as i16)?;
		writer.write_i32::<LittleEndian>(world.id)?;
		writer.write_varstring(&world.name)?;
		writer.write_u8(world.gamemode as u8)?;
		writer.write_u128::<LittleEndian>(world.uuid)?;
		writer.write_u64::<LittleEndian>(1)?; //idk what this is "World Generator Version" (seems to be 1 in packet analysis)
		writer.write_u8(world.style.moon_style)?; // ? Moon Type (set to 225 in packet analysis)
		
		writer.write_u8(world.style.forest_bg[0])?;
		writer.write_u8(world.style.corruption_bg)?;
		writer.write_u8(world.style.jungle_bg)?;
		writer.write_u8(world.style.snow_bg)?;
		writer.write_u8(world.style.hallow_bg)?;
		writer.write_u8(world.style.crimson_bg)?;
		writer.write_u8(world.style.desert_bg)?;
		writer.write_u8(world.style.ocean_bg)?;
		writer.write_u8(world.style.mushroom_bg)?;
		writer.write_u8(world.style.underworld_bg)?;
		writer.write(&world.style.forest_bg[1..=3])?;
		
		writer.write_u8(world.style.snow_bg_style)?;
		writer.write_u8(world.style.jungle_bg_style)?;
		writer.write_u8(world.style.underworld_bg_style)?;
		
		writer.write_f32::<LittleEndian>(world.weather.wind_speed)?;
		writer.write_u8(world.weather.num_clouds as u8)?;
		for i in world.style.forest_bg_x.iter() {
			writer.write_i32::<LittleEndian>(*i as i32)?;
		}
		writer.write(&world.style.forest_bg_style[..])?; // Tree Style 1-4
		
		for i in world.style.cave_bg_x.iter() {
			writer.write_i32::<LittleEndian>(*i as i32)?;
		}
		
		writer.write(&world.style.cave_bg_style[..])?;
		writer.write(&world.style.tree_tops[..])?;
		
		writer.write_f32::<LittleEndian>(world.weather.rain_amount)?;
		
		writer.write(&[0u8; 7][..])?; // Skipping event info
		
		writer.write_i16::<LittleEndian>(world.ore_tiers.copper)?;
		writer.write_i16::<LittleEndian>(world.ore_tiers.iron)?;
		writer.write_i16::<LittleEndian>(world.ore_tiers.silver)?;
		writer.write_i16::<LittleEndian>(world.ore_tiers.gold)?;
		writer.write_i16::<LittleEndian>(world.ore_tiers.cobalt)?;
		writer.write_i16::<LittleEndian>(world.ore_tiers.mythril)?;
		writer.write_i16::<LittleEndian>(world.ore_tiers.adamantite)?;
		
		writer.write_i8(world.events.invasion_type as i8)?; // Invasion Type (SByte)
		writer.write_u64::<LittleEndian>(0)?; //Lobby ID
		writer.write_f32::<LittleEndian>(world.weather.sandstorm_severity)?;
		
		info.data = writer;
		
		Ok(info)
	}
}