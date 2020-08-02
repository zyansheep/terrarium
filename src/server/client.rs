#![allow(unused_imports)]
#![allow(dead_code)]

use log::{trace, debug, info, warn, error};
use std::error::Error;
use std::sync::Arc;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock, Mutex};
use tokio::stream::{self, StreamExt};
use tokio_util::codec::{FramedRead, FramedWrite};
use futures::sink::SinkExt;
use arc_swap::ArcSwap;

use crate::server::{
	packet::{
		Packet, PacketCodec, PacketError, 
		types::{NetworkText}
	},
	ServerActionSender,
	ServerAction,
	player::Player,
};
use crate::world::*;

#[derive(Debug)]
pub enum ClientAction {
	SendPacket(Packet),
	UpdateChunkHandler(ChunkActionSender)
}
pub type ClientActionSender = mpsc::Sender<ClientAction>;
pub struct Client {
	pub id: usize, // What the client thinks its index is
	pub player: Player,
	pub action: ClientActionSender,
}
impl Client {
	pub fn new() -> (Self, mpsc::Receiver<ClientAction>) {
		let (action, action_receiver) = mpsc::channel(100);
		(Client {
			id: 0,
			player: Player::default(),
			action,
		}, action_receiver)
	}
	async fn send_packet(&mut self, packet: Packet) -> Result< (), mpsc::error::SendError<ClientAction> > {
		self.action.send( ClientAction::SendPacket(packet) ).await
	}
	pub async fn handle(
		&mut self, 
		socket: TcpStream, 
		mut action_receiver: mpsc::Receiver<ClientAction>, 
		mut server_action: ServerActionSender, 
		mut world_action: WorldActionSender,
		chunk_action: Arc<Mutex<ChunkActionSender>>) -> Result<(), Box<dyn Error>> {
		
		let (reader, writer) = tokio::io::split(socket);
		
		let chunk_action_updater = chunk_action.clone();
		// Broadcast thread
		tokio::spawn(async move {
			let mut packet_writer = FramedWrite::new(writer, PacketCodec::default());
			loop {
				if let Some(action) = action_receiver.recv().await {
					use ClientAction::*;
					debug!("Sending ClientAction: {:?}", action);
					let result = match action { // Parse action
						SendPacket(packet) => packet_writer.send(&packet).await,
						UpdateChunkHandler(handle) => { // Update chunk arcswap if needed
							let mut lock = chunk_action_updater.lock().await;
							*lock = handle; // Set new handle
							Ok(())
						},
					};
					if let Err(err) = result {
						error!("Error with parsing ClientAction: {:?}", err); break;
					}
				} else {
					break;
				}
			}
		});
		
		// Reader thread
		let mut packet_reader = FramedRead::new(reader, PacketCodec::default());
		loop {
			let result = packet_reader.try_next().await; // Try to Decode Packet
			
			if let Ok(packet_or_none) = result {
				if let Some(packet) = packet_or_none { // Check if packet was read
					debug!("Decoded Packet: {:?}", packet);
					
					use Packet::*;
					match packet {
						Packet::ConnectRequest(s) => {
							if s == "Terraria230"{
								self.send_packet(Packet::SetUserSlot(0)).await? // Every client is always in user slot 0 (other players are dynamically set up to 256 user slots)
							} else {
								self.send_packet(Packet::Disconnect(NetworkText::new("LegacyMultiplayer.4"))).await? // Send "Wrong Version" prompt
							}
						},
						Packet::PlayerAppearance(appearance) => self.player.appearance.init(appearance)?,
						Packet::PlayerUUID(s) => self.player.uuid = s,
						PlayerHp{..} | PlayerMana{..} | PlayerBuff{..} => self.player.status.init(packet)?,
						PlayerInventorySlot{..} => self.player.inventory.update_slot(packet)?, //TODO: Impl config flag to have server-side managed inventory (e.g. drop this packet)
						WorldDataRequest => world_action.send(WorldAction::RequestWorldInfo(self.action.clone())).await?, // Sends request to world thread to return cached worldinfo struct
						EssentialTilesRequest(_x, _y) => {
							self.send_packet(Packet::Status(15, NetworkText::new("LegacyInterface.44"), 0)).await?;
							// Request cached WorldInfo data from world
							let mut lock = chunk_action.lock().await;
							lock.send(ChunkAction::RequestSections(self.action.clone())).await?;
						}
						_ => warn!("Unimplemented Packet"), 
					}
				}else{ continue; }
			} else {
				error!("Error with reading packet: {:?}", result);
				break;
			}
		}
		server_action.send(ServerAction::DisconnectClient(self.id));
		Ok(())
	}
}