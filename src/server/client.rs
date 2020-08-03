#![allow(unused_imports)]
#![allow(dead_code)]

use log::{trace, debug, info, warn, error};
use std::error::Error;
use std::sync::Arc;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock, Mutex};
use tokio::stream::{self, StreamExt};
use tokio_util::codec::{FramedRead, FramedWrite};
use tokio::prelude::*;
use tokio::io::ReadHalf;

use futures::sink::SinkExt;
use arc_swap::ArcSwap;

use crate::server::{
	packet::{
		Packet, PacketCodec, PacketError, 
		types::{NetworkText}
	},
	ServerActionSender,
	ServerAction,
	player::*,
};
use crate::world::*;

#[derive(Debug)]
pub enum ClientAction {
	SetClientID(usize),
	SetPlayerName(String),
	SetPlayerAppearance(Appearance),
	SetPlayerUUID(String),
	SetPlayerStat(Packet),
	UpdateInventorySlot(Packet),
	
	RequestWorldInfo(),
	RequestEssentialTiles(TileCoord),
	
	SendPacket(Packet),
	UpdateChunkHandler(ChunkActionSender)
}
pub type ClientActionSender = mpsc::Sender<ClientAction>;
pub struct Client {
	pub id: usize, // What the client thinks its index is
	pub player: Player,
	pub action: ClientActionSender,
	connected_server: bool,
	connected_world: bool,
}
impl Client {
	pub fn new() -> (Self, mpsc::Receiver<ClientAction>) {
		let (action, action_receiver) = mpsc::channel(100);
		(Client {
			id: 0,
			player: Player::default(),
			action, 
			connected_server: false,
			connected_world: false
		}, action_receiver)
	}
	pub async fn handle_packets(mut action: ClientActionSender, reader: ReadHalf<TcpStream>) -> Result<(), Box<dyn Error>> {
		let mut packet_reader = FramedRead::new(reader, PacketCodec::default());
		
		loop {
			match packet_reader.try_next().await { 
				Ok(Some(packet)) => { // If data can be read and packet was parsed
					debug!("Decoded Packet: {:?}", packet);
					use ClientAction::*;
					match packet {
						Packet::ConnectRequest(s) => {
							if s == "Terraria230"{
								action.send(SendPacket(Packet::SetUserSlot(0))).await? // Every client is always in user slot 0 (other players are dynamically set up to 256 user slots by server thread)
							} else {
								action.send(SendPacket(Packet::Disconnect(NetworkText::new("LegacyMultiplayer.4")))).await? // Send "Wrong Version" prompt
							}
						},
						Packet::PlayerInfo(name, appearance) => {
							// Make sure these attributes can't be changed mid-game
							action.send(SetPlayerName(name)).await?;
							action.send(SetPlayerAppearance(appearance)).await?;
						},
						Packet::PlayerUUID(s) => action.send(SetPlayerUUID(s)).await?,
						Packet::PlayerHp{..} | Packet::PlayerMana{..} | Packet::PlayerBuff{..} => action.send(SetPlayerStat(packet)).await?,
						Packet::PlayerInventorySlot{..} => action.send(UpdateInventorySlot(packet)).await?,
						Packet::WorldDataRequest => action.send(RequestWorldInfo()).await?,
						Packet::EssentialTilesRequest(x, y) => {
							action.send(SendPacket(Packet::Status(15, NetworkText::new("LegacyInterface.44"), 0))).await?;
							// Request cached WorldInfo data from world
							action.send(RequestEssentialTiles( TileCoord{x: x as u16, y: y as u16} )).await?;
						}
						_ => warn!("Unimplemented Packet"), 
					}
				},
				Ok(None) => continue,
				Err(err) => {
					error!("Failed to parse packet: {:?}", err);
					break;
				}
			}
		}
		
		Ok(())
	}
	pub async fn handle(
		&mut self, 
		socket: TcpStream, 
		mut action_receiver: mpsc::Receiver<ClientAction>, 
		mut server_action: ServerActionSender, 
		mut world_action: WorldActionSender,
		chunk_action: Arc<Mutex<ChunkActionSender>>) -> Result<(), Box<dyn Error>> {
		
		let (reader, writer) = tokio::io::split(socket);
		
		// Packet Parsing Thread
		let packet_action = self.action.clone();
		tokio::spawn(async move {
			if let Err(err) = Client::handle_packets(packet_action, reader).await {
				error!("Packet Reading Thread errored: {:?}", err);
			}
		});
		
		// Action Thread
		let mut packet_writer = FramedWrite::new(writer, PacketCodec::default());
		loop {
			if let Some(action) = action_receiver.recv().await {
				use ClientAction::*;
				debug!("Sending ClientAction: {:?}", action);
				
				match action { // Parse action
					SendPacket(packet) => packet_writer.send(&packet).await?,
					SetPlayerName(name) => {
						if self.connected_server { continue; } // TODO: Maybe log double sends?
						
						self.player.name = name;
						// Server is notified of Client
						self.connected_server = true;
						server_action.send(ServerAction::ConnectClient(self.player.name.clone(), self.action.clone())).await?;
					},
					SetClientID(id) => {
						self.id = id;
					}
					UpdateInventorySlot(packet) => {
						//TODO: Implement config flag to have server-side managed inventory (e.g. drop this action)
						self.player.inventory.update_slot(packet)?;
					},
					SetPlayerAppearance(appearance) => self.player.appearance = appearance,
					
					RequestWorldInfo() => {
						world_action.send(WorldAction::RequestWorldInfo(self.action.clone())).await?;
					},
					RequestEssentialTiles(_coord) => {
						let mut lock = chunk_action.lock().await;
						lock.send(ChunkAction::RequestSections(self.action.clone())).await?;
					},
					
					UpdateChunkHandler(handle) => { // Update chunk arcswap if needed
						let mut lock = chunk_action.lock().await;
						*lock = handle; // Set new handle
					},
					_ => debug!("Unimplemented ClientAction: {:?}", action),
				}
			} else {
				debug!("All Senders have dropped for client: {:?}", self.player.name);
				break;
			}
		}
		
		server_action.send(ServerAction::DisconnectClient(self.player.name.clone())).await?;
		Ok(())
	}
}