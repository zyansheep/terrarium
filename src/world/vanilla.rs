use log::{debug, warn};
use std::{
	io::{self, Read},
	hash::{Hash, Hasher},
	convert::{TryFrom, TryInto},
	error::Error,
	collections::hash_map::DefaultHasher,
};

use byteorder::{LittleEndian, ReadBytesExt};
use variant_encoding::VarStringReader;
use crate::enum_primitive::FromPrimitive;

use crate::world::{
	chunk::{
		Sign, Tile, Chest, chest::ItemStack,
		tile::{Wall, WallType, Liquid, Block, TileCoord},
	},
	World, WorldMode, CorruptionType,
	//test::IronTier,
	world_types::{CopperTier, IronTier, SilverTier, GoldTier, CobaltTier, MythrilTier, AdamantiteTier},
};

/// Read a vanilla format world into a [World](../world/struct.World.html).
pub fn read(reader: &mut impl Read) -> Result<World, Box<dyn Error>> {
	let mut world = World::default();

	// File format header.
	debug!("Reading File Metadata");
	
	if reader.read_i32::<LittleEndian>()? != 230 { // Check Version
		return Err(Box::new(io::Error::new(io::ErrorKind::InvalidInput, "Wrong Vanilla World Version")));
	}

	// File metadata.
	reader.read_i64::<LittleEndian>()?; // Magic number and file type.
	reader.read_u32::<LittleEndian>()?; // Revision.
	reader.read_u64::<LittleEndian>()?; // Is favorite.

	// Chunk offsets.
	let chunk_count = reader.read_i16::<LittleEndian>()?;
	for _ in 0..chunk_count {
		reader.read_i32::<LittleEndian>()?;
	}

	// Tile frame important.
	let tile_count = reader.read_i16::<LittleEndian>()?;
	let mut tile_frame_important = vec![false; tile_count as usize];
	let mut byte = reader.read_u8()?;
	let mut bit = 0b00000001;
	for i in 0..tile_count as usize {
		tile_frame_important[i] = byte & bit != 0;
		if bit == 0b10000000 {
			byte = reader.read_u8()?;
			bit = 0b00000001;
		} else {
			bit <<= 1;
		}
	}

	// World header.
	debug!("Reading World Header");

	world.name = reader.read_varstring()?;
	
	let mut hasher = DefaultHasher::new();
	reader.read_varstring()?.hash(&mut hasher);
	world.gen_data.seed = hasher.finish();

	reader.read_u64::<LittleEndian>()?; // Generator version.
	world.uuid = reader.read_u128::<LittleEndian>()?;
	world.id = reader.read_i32::<LittleEndian>()?;

	world.dimensions.left = reader.read_i32::<LittleEndian>()? as u32;
	world.dimensions.right = reader.read_i32::<LittleEndian>()? as u32;
	world.dimensions.top = reader.read_i32::<LittleEndian>()? as u32;
	world.dimensions.bottom = reader.read_i32::<LittleEndian>()? as u32;

	world.dimensions.tile_height = reader.read_i32::<LittleEndian>()? as u32;
	world.dimensions.tile_width = reader.read_i32::<LittleEndian>()? as u32;

	world.worldmode = match reader.read_i32::<LittleEndian>()? {
		1 => WorldMode::Expert,
		2 => WorldMode::Master,
		3 => WorldMode::Journey,
		_ => WorldMode::Normal,
	};

	reader.read_u8()? != 0; // Is Drunk? (has seed "05162020")
	reader.read_u8()? != 0; // Has seed "for the worthy"

	reader.read_u64::<LittleEndian>()?; // Creation time.

	world.style.moon_style = reader.read_u8()?;

	for i in 0..3 {
		world.style.forest_bg_x[i] = reader.read_i32::<LittleEndian>()? as u32;
	}
	for i in 0..4 {
		world.style.forest_bg_style[i] = reader.read_i32::<LittleEndian>()? as u8;
	}

	for i in 0..3 {
		world.style.cave_bg_x[i] = reader.read_i32::<LittleEndian>()? as u32;
	}
	for i in 0..4 {
		world.style.cave_bg_style[i] = reader.read_i32::<LittleEndian>()? as u8;
	}

	world.style.snow_bg_style = reader.read_i32::<LittleEndian>()? as u8;
	world.style.jungle_bg_style = reader.read_i32::<LittleEndian>()? as u8;
	world.style.underworld_bg_style = reader.read_i32::<LittleEndian>()? as u8;
	
	world.spawn_coord = TileCoord {
		x: reader.read_i32::<LittleEndian>()? as u16,
		y: reader.read_i32::<LittleEndian>()? as u16
	};

	world.gen_data.surface_y = reader.read_f64::<LittleEndian>()? as u32;
	world.gen_data.rock_layer_y = reader.read_f64::<LittleEndian>()? as u32;

	world.time.time = reader.read_f64::<LittleEndian>()? as u32;
	world.time.is_day = reader.read_u8()? != 0;

	world.time.moon_phase = reader.read_i32::<LittleEndian>()? as u8;

	world.events.is_blood_moon_happening = reader.read_u8()? != 0;
	world.events.is_eclipse_happening = reader.read_u8()? != 0;

	world.dungeon_coord = TileCoord {
		x: reader.read_i32::<LittleEndian>()? as u16,
		y: reader.read_i32::<LittleEndian>()? as u16
	};
	
	world.gen_data.corruption_type = match reader.read_u8()? {
		0 => CorruptionType::Corruption,
		_ => CorruptionType::Crimson,
	};

	world.progress.defeated_eye_of_cthulu = reader.read_u8()? != 0;
	world.progress.defeated_eater_or_brain = reader.read_u8()? != 0;
	world.progress.defeated_skeletron = reader.read_u8()? != 0;

	world.progress.defeated_queen_bee = reader.read_u8()? != 0;

	world.progress.defeated_destroyer = reader.read_u8()? != 0;
	world.progress.defeated_twins = reader.read_u8()? != 0;
	world.progress.defeated_skeletron_prime = reader.read_u8()? != 0;
	reader.read_u8()?; // Downed any mech boss.

	world.progress.defeated_plantera = reader.read_u8()? != 0;
	world.progress.defeated_golem = reader.read_u8()? != 0;

	world.progress.defeated_king_slime = reader.read_u8()? != 0;

	world.progress.saved_goblin = reader.read_u8()? != 0;
	world.progress.saved_wizard = reader.read_u8()? != 0;
	world.progress.saved_mechanic = reader.read_u8()? != 0;

	world.progress.defeated_goblin_army = reader.read_u8()? != 0;

	world.progress.defeated_clown = reader.read_u8()? != 0;

	world.progress.defeated_frost_legion = reader.read_u8()? != 0;
	world.progress.defeated_pirates = reader.read_u8()? != 0;

	let broken_shadow_orb = reader.read_u8()? != 0;

	world.events.is_meteor_queued = reader.read_u8()? != 0;

	world.progress.shadow_orbs_broken = reader.read_u8()?;
	if broken_shadow_orb && world.progress.shadow_orbs_broken == 0 {
		world.progress.shadow_orbs_broken = 3;
	}
	world.progress.altars_broken = reader.read_i32::<LittleEndian>()? as u32;

	world.progress.is_hardmode = reader.read_u8()? != 0;

	world.events.invasion_delay = reader.read_i32::<LittleEndian>()?;
	world.events.invasion_size = reader.read_i32::<LittleEndian>()?;
	world.events.invasion_type = reader.read_i32::<LittleEndian>()? as u8;
	world.events.invasion_x = reader.read_f64::<LittleEndian>()? as u32;

	world.events.slime_rain_remaining_time = reader.read_f64::<LittleEndian>()? as i32; // version > 118

	world.time.sundial_cooldown = reader.read_u8()?; // version > 113

	world.weather.is_raining = reader.read_u8()? != 0;
	world.weather.rain_time = reader.read_i32::<LittleEndian>()? as u32;
	world.weather.rain_amount = reader.read_f32::<LittleEndian>()?;
	
	world.gen_data.cobalt_tier = CobaltTier::try_from(reader.read_i32::<LittleEndian>()? as u16)?;
	world.gen_data.mythril_tier = MythrilTier::try_from(reader.read_i32::<LittleEndian>()? as u16)?;
	world.gen_data.adamantite_tier = AdamantiteTier::try_from(reader.read_i32::<LittleEndian>()? as u16)?;

	world.style.forest_bg[0] = reader.read_u8()?;
	world.style.corruption_bg = reader.read_u8()?;
	world.style.jungle_bg = reader.read_u8()?;
	world.style.snow_bg = reader.read_u8()?;
	world.style.hallow_bg = reader.read_u8()?;
	world.style.crimson_bg = reader.read_u8()?;
	world.style.desert_bg = reader.read_u8()?;
	world.style.ocean_bg = reader.read_u8()?;

	world.weather.active_cloud_bg = reader.read_i32::<LittleEndian>()?;
	world.weather.num_clouds = reader.read_i16::<LittleEndian>()? as u16;

	world.weather.wind_speed = reader.read_f32::<LittleEndian>()?;

	world
		.angler
		.completed_quests
		.reserve_exact(reader.read_i32::<LittleEndian>()? as usize);
	for _ in 0..world.angler.completed_quests.capacity() {
		world.angler.completed_quests.push(reader.read_varstring()?);
	}
	world.progress.saved_angler = reader.read_u8()? != 0;
	world.angler.quest = reader.read_i32::<LittleEndian>()? as u8;

	world.progress.saved_stylist = reader.read_u8()? != 0;
	world.progress.saved_tax_collector = reader.read_u8()? != 0;
	world.progress.saved_golfer = reader.read_u8()? != 0;

	world.events.invasion_start_size = reader.read_i32::<LittleEndian>()?;

	world.events.cultist_remaining_time = reader.read_i32::<LittleEndian>()?;

	world.progress.entity_kill_counts.reserve_exact(reader.read_i16::<LittleEndian>()? as usize);
	for _ in 0..world.progress.entity_kill_counts.capacity() {
		world.progress.entity_kill_counts.push(reader.read_i32::<LittleEndian>()? as u32);
	}

	world.time.is_fast_forwarding = reader.read_u8()? != 0;

	world.progress.defeated_duke_fishron = reader.read_u8()? != 0;
	world.progress.defeated_martians = reader.read_u8()? != 0;
	world.progress.defeated_lunatic_cultist = reader.read_u8()? != 0;
	world.progress.defeated_moon_lord = reader.read_u8()? != 0;
	world.progress.defeated_pumpking = reader.read_u8()? != 0;
	world.progress.defeated_mourning_wood = reader.read_u8()? != 0;
	world.progress.defeated_ice_queen = reader.read_u8()? != 0;
	world.progress.defeated_santa_nk1 = reader.read_u8()? != 0;
	world.progress.defeated_everscream = reader.read_u8()? != 0;
	world.progress.defeated_solar_pillar = reader.read_u8()? != 0;
	world.progress.defeated_vortex_pillar = reader.read_u8()? != 0;
	world.progress.defeated_nebula_pillar = reader.read_u8()? != 0;
	world.progress.defeated_stardust_pillar = reader.read_u8()? != 0;

	world.events.is_solar_pillar_alive = reader.read_u8()? != 0;
	world.events.is_vortex_pillar_alive = reader.read_u8()? != 0;
	world.events.is_nebula_pillar_alive = reader.read_u8()? != 0;
	world.events.is_stardust_pillar_alive = reader.read_u8()? != 0;
	world.events.is_impending_doom_approaching = reader.read_u8()? != 0;

	world.events.is_party_manual = reader.read_u8()? != 0;
	world.events.is_party_genuine = reader.read_u8()? != 0;
	world.events.party_cooldown = reader.read_i32::<LittleEndian>()? as u8;
	world
		.events
		.partying_npcs
		.reserve_exact(reader.read_i32::<LittleEndian>()? as usize);
	for _ in 0..world.events.partying_npcs.capacity() {
		world
			.events
			.partying_npcs
			.push(reader.read_i32::<LittleEndian>()? as u32);
	}

	world.weather.is_sandstorm_happening = reader.read_u8()? != 0;
	world.weather.sandstorm_remaining_time = reader.read_i32::<LittleEndian>()?;
	world.weather.sandstorm_severity = reader.read_f32::<LittleEndian>()?;
	world.weather.sandstorm_intended_severity = reader.read_f32::<LittleEndian>()?;

	world.progress.saved_bartender = reader.read_u8()? != 0;

	world.progress.defeated_old_ones_army_tier_1 = reader.read_u8()? != 0;
	world.progress.defeated_old_ones_army_tier_2 = reader.read_u8()? != 0;
	world.progress.defeated_old_ones_army_tier_3 = reader.read_u8()? != 0;

	world.style.mushroom_bg = reader.read_u8()?;
	world.style.underworld_bg = reader.read_u8()?;
	world.style.forest_bg[1] = reader.read_u8()?;
	world.style.forest_bg[2] = reader.read_u8()?;
	world.style.forest_bg[3] = reader.read_u8()?;

	world.progress.used_combat_book = reader.read_u8()? != 0;

	world.events.lantern_night_cooldown = reader.read_i32::<LittleEndian>()? as u8;
	reader.read_u8()?; // Is lantern night genuine.
	reader.read_u8()?; // Is lantern night manual.
	world.events.is_lantern_night_queued = reader.read_u8()? != 0;

	for i in 0..reader.read_i32::<LittleEndian>()? as usize {
		world.style.tree_tops[i] = reader.read_i32::<LittleEndian>()? as u8;
	}

	world.events.is_halloween_forced = reader.read_u8()? != 0;
	world.events.is_christmas_forced = reader.read_u8()? != 0;

	world.gen_data.copper_tier = CopperTier::try_from(reader.read_i32::<LittleEndian>()? as u16)?;
	world.gen_data.iron_tier = IronTier::try_from(reader.read_i32::<LittleEndian>()? as u16)?;
	world.gen_data.silver_tier = SilverTier::try_from(reader.read_i32::<LittleEndian>()? as u16)?;
	world.gen_data.gold_tier = GoldTier::try_from(reader.read_i32::<LittleEndian>()? as u16)?;

	world.progress.purchased_cat = reader.read_u8()? != 0;
	world.progress.purchased_dog = reader.read_u8()? != 0;
	world.progress.purchased_bunny = reader.read_u8()? != 0;

	world.progress.defeated_empress_of_light = reader.read_u8()? != 0;
	world.progress.defeated_queen_slime = reader.read_u8()? != 0;

	// Tiles.
	debug!("reading tiles");
	
	let mut tiles = Vec::with_capacity(
		world.dimensions.tile_width as usize * world.dimensions.tile_height as usize,
	);
	
	for x in 0..world.dimensions.tile_width {
		let mut y = 0;
		while y < world.dimensions.tile_height {
			let mut tile = Tile::default();
			
			let current_coord = TileCoord { x: x as u16, y: y as u16 };
			let flags1 = reader.read_u8()?;
			
			if (flags1 & 0b00000010) != 0 {
				tile.block = Some(Block::default());
			}

			if (flags1 & 0b00000100) != 0 {
				tile.wall = Some(Wall::default());
			}

			let liquid_id = (flags1 & 0b00011000) >> 3;
			if liquid_id != 0 { // check if liquid
				tile.liquid = Some(Liquid {
					id: liquid_id,
					amount: 0,
				});
			}

			let mut is_block_colored = false;
			let mut is_wall_colored = false;
			let mut is_wall_id_u16 = false;

			if (flags1 & 0b00000001) != 0 {
				let flags2 = reader.read_u8()?; // Block metadata flags
				
				if let Some(mut block) = &mut tile.block { block.slope = (flags2 & 0b01110000) >> 4 } // Set slope

				// Set wire states
				tile.has_wire[0] = (flags2 & 0b00000010) != 0;
				tile.has_wire[1] = (flags2 & 0b00000100) != 0;
				tile.has_wire[2] = (flags2 & 0b00001000) != 0;

				if (flags2 & 0b00000001) != 0 {
					let flags3 = reader.read_u8()?;
					
					if let Some(block) = &mut tile.block { block.is_inactive = (flags3 & 0b00000100) != 0; }

					tile.has_actuator = (flags3 & 0b00000010) != 0;
					is_block_colored = (flags3 & 0b00001000) != 0;
					is_wall_colored = (flags3 & 0b00010000) != 0;
					tile.has_wire[3] = (flags3 & 0b00100000) != 0;
					is_wall_id_u16 = (flags3 & 0b01000000) != 0;
				}
			}
			
			if let Some(block) = &mut tile.block {
				if (flags1 & 0b00100000) != 0 {
					block.id = reader.read_u16::<LittleEndian>()?;
				} else {
					block.id = reader.read_u8()? as u16;
				}

				if tile_frame_important[block.id as usize] {
					block.frame_width = reader.read_u16::<LittleEndian>()?;
					block.frame_height = reader.read_u16::<LittleEndian>()?;
				}

				if is_block_colored {
					block.color = reader.read_u8()?;
				}
			}
			
			if let Some(wall) = &mut tile.wall {
				if let Some(id) = WallType::from_u16(reader.read_u8()? as u16) { wall.id = id }
				else { warn!("Failed to read Wall ID"); }
				if is_wall_colored {
					wall.color = reader.read_u8()?;
				}
			}
			
			if let Some(liquid) = &mut tile.liquid {
				liquid.amount = reader.read_u8()?;
			}
			
			if let Some(wall) = &mut tile.wall {
				if is_wall_id_u16 {
					let mut read_id = wall.id as u16;
					read_id |= (reader.read_u8()? as u16) << 8;
					if let Some(id) = WallType::from_u16(read_id) { wall.id = id; }
					else { warn!("Failed to read Wall ID (type u16) found {:?} at tile: {:?}", read_id, current_coord); }
				}
			}

			let mut repeat = 0;
			if (flags1 & 0b10000000) != 0 {
				repeat = reader.read_u16::<LittleEndian>()? as u32;
			} else if (flags1 & 0b01000000) != 0 {
				repeat = reader.read_u8()? as u32;
			}

			y += repeat + 1;

			for _ in 0..repeat + 1 {
				tiles.push(tile);
			}
		}
	}

	// Chests.
	debug!("Reading Chests");
	
	let mut chests = Vec::with_capacity(reader.read_i16::<LittleEndian>()? as usize);

	let item_count = reader.read_i16::<LittleEndian>()? as usize;

	for _ in 0..chests.capacity() {
		let mut chest = Chest::default();
		chest.x = reader.read_i32::<LittleEndian>()? as u32;
		chest.y = reader.read_i32::<LittleEndian>()? as u32;
		chest.name = reader.read_varstring()?;

		chest.items.reserve_exact(item_count);
		for _ in 0..item_count {
			let stack = reader.read_i16::<LittleEndian>()? as u16;
			if stack > 0 {
				chest.items.push(ItemStack {
					stack: stack,
					id: reader.read_i32::<LittleEndian>()? as u16,
					prefix: reader.read_u8()?,
				});
			} else {
				chest.items.push(ItemStack {stack: 0, id: 0, prefix: 0});
			}
		}

		chests.push(chest);
	}
	
	// Signs.
	debug!("Reading Signs");

	let mut signs = Vec::with_capacity(reader.read_i16::<LittleEndian>()? as usize);
	
	for _ in 0..signs.capacity() {
		let mut sign = Sign::default();

		sign.text = reader.read_varstring()?;
		sign.x = reader.read_i32::<LittleEndian>()? as u32;
		sign.y = reader.read_i32::<LittleEndian>()? as u32;

		signs.push(sign);
	}
	let chunk_percent = 0.0;
	debug!("Creating Chunks: {}", chunk_percent);

	Ok(world)
}
