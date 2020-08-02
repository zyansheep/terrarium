
use rand::Rng;
use crate::world::{
	world_types::{CorruptionType},
	GenerationData,
	chunk::{Chunk, ChunkCoord, TileCoord},
};

pub trait WorldGenerator {
	fn calculate_gen_data(&mut self) -> GenerationData;
	fn generate(&mut self, seed: u64) -> Vec<Chunk>;
}

pub enum WorldSize {
	Small,
	Medium,
	Large,
}

pub struct NormalGen<R: Rng> {
	size: TileCoord,
	corruption_type: CorruptionType,
	rng: R,
}
impl<R: Rng> WorldGenerator for NormalGen<R> {
	fn calculate_gen_data(&mut self) -> GenerationData {
		let rng = &mut self.rng;
		GenerationData {
			seed: rng.gen(),
			corruption_type: self.corruption_type,
			surface_y: rng.gen_range(100, 200),
			rock_layer_y: rng.gen_range(3000, 3100),
			
			copper_tier: rng.gen(),
			iron_tier: rng.gen(),
			silver_tier: rng.gen(),
			gold_tier: rng.gen(),
			cobalt_tier: rng.gen(),
			mythril_tier: rng.gen(),
			adamantite_tier: rng.gen(),
		}
	}
	fn generate(&mut self, seed: u64) -> Vec<Chunk> {
		Vec::with_capacity(3)
	}
}
impl<R: Rng> NormalGen<R> {
	pub fn new(world_size: WorldSize, mut rng: R, corruption_type: Option<CorruptionType>) -> NormalGen<R> {
		NormalGen {
			size: match world_size {
				Small => TileCoord {x: 4200, y: 1200},
				Medium => TileCoord {x: 6200, y :1800},
				Large => TileCoord {x: 8200, y: 2400},
			},
			corruption_type: {
				if let Some(c_type) = corruption_type { c_type } else { rng.gen() }
			},
			rng: rng,
		}
	}
}