#![allow(dead_code)]

use log::{trace, error};
use std::{io::{self, Read, Write}, error::Error, sync::Arc};
use serde::{Serialize, Deserialize};
use tokio::sync::{mpsc, Mutex, OwnedMutexGuard};
use arc_swap::ArcSwap;
use flate2::{Compression, read::DeflateDecoder, write::DeflateEncoder};

pub mod tile;
pub mod chest;
pub mod sign;
pub use tile::{TileCoord, Tile, Block};
pub use chest::Chest;
pub use sign::Sign;

use crate::server::ClientActionSender;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ChunkCoord {
	pub x: u16,
	pub y: u16,
}
impl ChunkCoord {
	pub fn from_tilecoord(coord: &TileCoord, chunk_size: u16) -> ChunkCoord {
		ChunkCoord { x: coord.x / chunk_size, y: coord.y / chunk_size }
	}
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Chunk {
	pub start_pos: TileCoord,
	pub chunk_size: u16,
	
	pub tiles: Vec<Tile>,
	pub chests: Vec<Chest>,
	pub signs: Vec<u8>,
	pub tileentities: Vec<u8>,
}

impl Chunk {
	pub fn test_chunk(chunk_size: u16) -> Chunk {
		let mut chunk = Chunk::default();
		chunk.tiles.reserve_exact( chunk_size as usize * chunk_size as usize);
		for _ in 0..chunk_size {
			for x in 0..chunk_size {
				if x > chunk_size-3 {
					chunk.tiles.push(Tile::new(Block::new(0)));
				} else {
					chunk.tiles.push(Tile::default())
				}
			}
		}
		chunk
	}
	pub fn read(reader: &mut impl Read) -> Result<Chunk, Box<dyn Error>> {
		let mut reader = DeflateDecoder::new(reader);
		
		trace!("Reading Chunk");
		Ok(bincode::deserialize_from(&mut reader)?)
	}
	pub fn write(&self, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
		trace!("Compressing World Data");
		let mut writer = DeflateEncoder::new(writer, Compression::default());
		
		trace!("Writing Chunk");
		bincode::serialize_into(&mut writer, self)?;
		writer.finish()?;

		Ok(())
	}
}
#[derive(Debug)]
pub enum ChunkAction {
	AssignChunk(OwnedMutexGuard<Chunk>),
	RequestSections(ClientActionSender), // Send cached chunk packets to client
	
	ModifyBlock(ClientActionSender, TileCoord),
	ForceCloseThread(),
}
pub type ChunkActionSender = mpsc::Sender<ChunkAction>;

/// Holds locks on multiple chunks and manages client block interactions
pub struct ChunkThread {
	client_pool: Vec<ClientActionSender>,
	chunks: Vec<OwnedMutexGuard<Chunk>>,
}
impl ChunkThread {
	pub fn new(initial_chunk: OwnedMutexGuard<Chunk>) -> Self {
		ChunkThread {
			client_pool: Default::default(),
			chunks: vec![initial_chunk],
		}
	}
	pub async fn handle(&mut self, mut action_receiver: mpsc::Receiver<ChunkAction>) -> Result<(), Box<dyn Error>> {
		loop {
			let result = action_receiver.recv().await;
			if let Some(action) = result {
				use ChunkAction::*;
				match action {
					RequestSections(_sender) => {
						// Send cached chunk data to client (generate if needed)
					},
					AssignChunk(chunk_lock) => { // Assigns a loaded chunk to this Chunk Thread (managed by World thread)
						self.chunks.push(chunk_lock);
					},
					ModifyBlock(_sender, _tile_coord) => {
						
					},
					ForceCloseThread() => break,
				}
			}
		}
		Ok(())
	}
}

/// Object that stores loaded chunk mutex
#[derive(Default)]
pub struct LoadedChunk {
	chunk: Arc<Mutex<Chunk>>, // Contains Chunk Mutex if chunk is loaded
	action: Option<ChunkActionSender>, // Contains action channel to chunk thread that has lock on this chunk
}
impl LoadedChunk {
	pub fn from_file(reader: impl io::BufRead) -> Result<Self, Box<dyn Error>> {
		let chunk = bincode::deserialize_from(reader)?;
		Ok(LoadedChunk {
			chunk: Arc::new(Mutex::new(chunk)),
			action: None,
		})
	}
	pub fn from_chunk(source: Chunk) -> Self {
		LoadedChunk {
			chunk: Arc::new(Mutex::new(source)),
			action: None,
		}
	}
	pub async fn get_chunk_handle(&mut self) -> Result<ChunkActionSender, tokio::sync::TryLockError> {
		if let Some(action) = &self.action {
			Ok(action.clone())
		} else {
			let arc = self.chunk.clone();
			let lock = arc.try_lock_owned()?;
			let mut thread = ChunkThread::new(lock);
			let (tx, rx) = mpsc::channel::<ChunkAction>(100);
			
			tokio::spawn(async move {
				if let Err(err) = thread.handle(rx).await {
					error!("Chunk Thread Exited with error: {:?}", err);
				}
			});
			self.action = Some(tx.clone());
			Ok(tx)
		}
	}
	pub async fn send_lock(&mut self, sender: &mut ChunkActionSender) -> Result<(), Box<dyn Error>> {
		let arc = self.chunk.clone();
		let lock = arc.try_lock_owned()?;
		self.action = Some(sender.clone());
		sender.send(ChunkAction::AssignChunk(lock)).await?;
		Ok(())
	}
}