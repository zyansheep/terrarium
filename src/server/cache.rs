
use byteorder::{LittleEndian, WriteBytesExt};
use byteorder::ByteOrder;
use variant_encoding::VarStringWriter;
use std::sync::Arc;
use std::io::Write;
use std::error::Error;

use crate::world::World;

//Server Cache object, stores cached packets
#[derive(Default, Debug)]
pub struct Cache {
	pub world_info: Vec<u8>
}

impl Cache {
	pub fn construct_world_info_packet(&mut self, world: Arc<World>) -> Result<(), Box<dyn Error>> {
		self.world_info = Vec::with_capacity(22 + world.name.len() + 148);
		let mut writer = self.world_info.as_mut_slice();
		
		writer.write_i32::<LittleEndian>(world.time.time as i32)?; // Time
		if world.events.is_blood_moon_happening { // Day and Moon Info (1 = Day Time, 2 = Blood Moon, 4 = Eclipse)
			writer.write_u8(2)?;
		} else if world.events.is_eclipse_happening {
			writer.write_u8(4)?;
		} else {
			writer.write_u8(1)?;
		}

		writer.write_u8(world.time.moon_phase)?; // Moon Phase
		
		writer.write_i16::<LittleEndian>(world.dimensions.right as i16)?; // Max Tiles X
		writer.write_i16::<LittleEndian>(world.dimensions.bottom as i16)?; // Max Tiles Y
		writer.write_i16::<LittleEndian>(world.spawn_x as i16)?; // Spawn X
		writer.write_i16::<LittleEndian>(world.spawn_y as i16)?; // Spawn Y
		writer.write_i16::<LittleEndian>(world.surface_y as i16)?;
		writer.write_i16::<LittleEndian>(world.rock_layer_y as i16)?;
		writer.write_i32::<LittleEndian>(world.id)?;
		writer.write_varstring(&world.name)?;
		writer.write_u8(world.gamemode as u8)?;
		writer.write_u128::<LittleEndian>(world.uuid)?;
		writer.write_u64::<LittleEndian>(230)?; //idk what this is "World Generator Version"
		writer.write_u8(world.style.moon_style)?; // ? Moon Type
		
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
		writer.write(&world.style.forest_bg[1..3])?;
		writer.write_u8(world.style.snow_bg_style)?;
		writer.write_u8(world.style.jungle_bg_style)?;
		writer.write_u8(world.style.underworld_bg_style)?;
		
		writer.write_f32::<LittleEndian>(world.weather.wind_speed)?;
		writer.write_u8(world.weather.num_clouds as u8)?;
		LittleEndian::write_u32_into(&world.style.forest_bg_x[..], writer); // Tree 1-3
		writer.write(&world.style.forest_bg_style[..])?; // Tree Style 1-4
		LittleEndian::write_u32_into(&world.style.cave_bg_x[..], writer);
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
		
		Ok(())
	}
}