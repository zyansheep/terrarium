
use rand::{distributions::{Distribution, Standard}, Rng};
use serde::{Serialize, Deserialize};
use num_enum::TryFromPrimitive;
use arc_swap::ArcSwap;

use rand_enum_derive::EnumRand;

pub type WorldCache = ArcSwap<Vec<u8>>;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Dimensions {
	pub left: u32,
	pub right: u32,
	pub top: u32,
	pub bottom: u32,

	pub tile_width: u32,
	pub tile_height: u32,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub enum WorldMode {
	Normal,
	Expert,
	Master,
	Journey,
}
impl Default for WorldMode { fn default() -> WorldMode { WorldMode::Normal } }

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Style {
	pub moon_style: u8,

	pub forest_bg_x: [u32; 3],
	pub forest_bg_style: [u8; 4],

	pub cave_bg_x: [u32; 3],
	pub cave_bg_style: [u8; 4],

	pub snow_bg_style: u8,
	pub jungle_bg_style: u8,
	pub underworld_bg_style: u8,

	pub forest_bg: [u8; 4],
	pub corruption_bg: u8,
	pub jungle_bg: u8,
	pub snow_bg: u8,
	pub hallow_bg: u8,
	pub crimson_bg: u8,
	pub desert_bg: u8,
	pub ocean_bg: u8,
	pub mushroom_bg: u8,
	pub underworld_bg: u8,

	pub tree_tops: [u8; 13],
}
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum CorruptionType {
	Corruption,
	Crimson,
}
impl Distribution<CorruptionType> for Standard {
	fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> CorruptionType {
		match rng.gen_range(0, 1) { // Need size of enum
			0 => CorruptionType::Corruption, // Iterate through things
			_ => CorruptionType::Crimson,
		}
	}
}
pub mod ore_tiers {
	use serde::{Serialize, Deserialize};
	use num_enum::TryFromPrimitive;
	use rand_enum_derive::EnumRand;
	#[derive(Debug, Deserialize, Serialize, TryFromPrimitive, EnumRand, Clone, Copy)]
	#[repr(u16)]
	pub enum CopperTier {
		CopperOre = 7,
		TinOre = 166,
	}
	#[derive(Debug, Deserialize, Serialize, TryFromPrimitive, EnumRand, Clone, Copy)]
	#[repr(u16)]
	pub enum IronTier {
		IronOre = 6,
		LeadOre = 167,
	}
	//use crate::world::test::IronTier;
	#[derive(Debug, Deserialize, Serialize, TryFromPrimitive, EnumRand, Clone, Copy)]
	#[repr(u16)]
	pub enum SilverTier {
		SilverOre = 9,
		TungstenOre = 168,
	}
	#[derive(Debug, Deserialize, Serialize, TryFromPrimitive, EnumRand, Clone, Copy)]
	#[repr(u16)]
	pub enum GoldTier {
		GoldOre = 8,
		PlatinumOre = 166,
	}
	#[derive(Debug, Deserialize, Serialize, TryFromPrimitive, EnumRand, Clone, Copy)]
	#[repr(u16)]
	pub enum CobaltTier {
		CobaltOre = 107,
		PalladiumOre = 221,
	}
	#[derive(Debug, Deserialize, Serialize, TryFromPrimitive, EnumRand, Clone, Copy)]
	#[repr(u16)]
	pub enum MythrilTier {
		MythilTier = 108,
		OrichalcumOre = 222,
	}
	#[derive(Debug, Deserialize, Serialize, TryFromPrimitive, EnumRand, Clone, Copy)]
	#[repr(u16)]
	pub enum AdamantiteTier {
		AdamantiteTier = 111,
		TitaniumTier = 223,
	}
}
pub use ore_tiers::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct GenerationData {
	pub seed: u64,
	pub corruption_type: CorruptionType,
	pub surface_y: u32,
	pub rock_layer_y: u32,
	
	pub copper_tier: CopperTier, // else tin
	pub iron_tier: IronTier, // else lead
	pub silver_tier: SilverTier, // else tungsten
	pub gold_tier: GoldTier, // else platinum
	pub cobalt_tier: CobaltTier, // else
	pub mythril_tier: MythrilTier, // else orichalcum
	pub adamantite_tier: AdamantiteTier, // else titanium
}
impl Default for GenerationData {
	fn default() -> GenerationData {
		GenerationData {
			seed: 0,
			corruption_type: CorruptionType::Corruption,
			surface_y: 0, rock_layer_y: 0,
			copper_tier: CopperTier::CopperOre, iron_tier: IronTier::IronOre, silver_tier: SilverTier::SilverOre, gold_tier: GoldTier::GoldOre,
			cobalt_tier: CobaltTier::CobaltOre, mythril_tier: MythrilTier::MythilTier, adamantite_tier: AdamantiteTier::AdamantiteTier,
		}
	}
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Progress {
	// Bosses.
	pub is_hardmode: bool,

	pub defeated_clown: bool,
	pub defeated_destroyer: bool,
	pub defeated_duke_fishron: bool,
	pub defeated_eater_or_brain: bool,
	pub defeated_empress_of_light: bool,
	pub defeated_everscream: bool,
	pub defeated_eye_of_cthulu: bool,
	pub defeated_golem: bool,
	pub defeated_ice_queen: bool,
	pub defeated_king_slime: bool,
	pub defeated_lunatic_cultist: bool,
	pub defeated_moon_lord: bool,
	pub defeated_mourning_wood: bool,
	pub defeated_nebula_pillar: bool,
	pub defeated_plantera: bool,
	pub defeated_pumpking: bool,
	pub defeated_queen_bee: bool,
	pub defeated_queen_slime: bool,
	pub defeated_santa_nk1: bool,
	pub defeated_skeletron: bool,
	pub defeated_skeletron_prime: bool,
	pub defeated_solar_pillar: bool,
	pub defeated_stardust_pillar: bool,
	pub defeated_twins: bool,
	pub defeated_vortex_pillar: bool,

	// Invasions.
	pub defeated_frost_legion: bool,
	pub defeated_goblin_army: bool,
	pub defeated_martians: bool,
	pub defeated_old_ones_army_tier_1: bool,
	pub defeated_old_ones_army_tier_2: bool,
	pub defeated_old_ones_army_tier_3: bool,
	pub defeated_pirates: bool,

	// Saved NPCs.
	pub saved_angler: bool,
	pub saved_bartender: bool,
	pub saved_goblin: bool,
	pub saved_golfer: bool,
	pub saved_mechanic: bool,
	pub saved_stylist: bool,
	pub saved_tax_collector: bool,
	pub saved_wizard: bool,

	// Town pets.
	pub purchased_cat: bool,
	pub purchased_dog: bool,
	pub purchased_bunny: bool,

	// Other.
	pub shadow_orbs_broken: u8,
	pub altars_broken: u32,

	pub used_combat_book: bool,
	
	pub entity_kill_counts: Vec<u32>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Time {
	pub is_day: bool,
	pub time: u32,

	pub moon_phase: u8,

	pub is_fast_forwarding: bool,
	pub sundial_cooldown: u8,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Weather {
	pub wind_speed: f32,

	// Clouds.
	pub active_cloud_bg: i32,
	pub num_clouds: u16,

	// Rain.
	pub is_raining: bool,
	pub rain_time: u32,
	pub rain_amount: f32,

	// Sandstorm.
	pub is_sandstorm_happening: bool,
	pub sandstorm_remaining_time: i32,
	pub sandstorm_severity: f32,
	pub sandstorm_intended_severity: f32,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Events {
	pub slime_rain_remaining_time: i32,

	pub is_blood_moon_happening: bool,

	pub is_meteor_queued: bool,

	pub is_eclipse_happening: bool,

	pub is_halloween_forced: bool,
	pub is_christmas_forced: bool,

	pub cultist_remaining_time: i32,

	// Invasion.
	pub invasion_type: u8,
	pub invasion_delay: i32,
	pub invasion_start_size: i32,
	pub invasion_size: i32,
	pub invasion_x: u32,

	// Lantern night.
	pub lantern_night_cooldown: u8,
	pub is_lantern_night_queued: bool,

	// Lunar events.
	pub is_solar_pillar_alive: bool,
	pub is_vortex_pillar_alive: bool,
	pub is_nebula_pillar_alive: bool,
	pub is_stardust_pillar_alive: bool,
	pub is_impending_doom_approaching: bool,

	// Party.
	pub is_party_manual: bool,
	pub is_party_genuine: bool,
	pub party_cooldown: u8,
	pub partying_npcs: Vec<u32>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Angler {
	pub completed_quests: Vec<String>,
	pub quest: u8,
}