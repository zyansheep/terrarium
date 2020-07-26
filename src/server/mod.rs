#![allow(dead_code)]
#![allow(unused_imports)]

use log::{trace, debug, info, warn, error};
use std::error::Error;
use std::sync::Arc;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock, Mutex};
use tokio::stream::{self, StreamExt};
use tokio_util::codec::{FramedRead, FramedWrite};
use futures::sink::SinkExt;

use crate::packet::{Packet, PacketCodec, PacketError, types::NetworkText};
use crate::world::*;
use crate::player::Player;


#[derive(Debug)]
pub enum ClientAction {
	SendPacket(Packet), 
	SetChunkHandler(ChunkActionSender)
}
pub type ClientActionSender = mpsc::Sender<Arc<ClientAction>>;
struct Client {
	id: usize, // What the client thinks its index is
	player: Player,
	action: ClientActionSender,
}
impl Client {
	fn new() -> (Self, mpsc::Receiver<Arc<ClientAction>>) {
		let (action, action_receiver) = mpsc::channel(100);
		(Client {
			id: 0,
			player: Player::default(),
			action,
		}, action_receiver)
	}
	async fn send_packet(&mut self, packet: Packet) -> Result< (), mpsc::error::SendError<Arc<ClientAction>> > {
		self.action.send( Arc::new(ClientAction::SendPacket(packet)) ).await
	}
	async fn handle(
		&mut self, 
		socket: TcpStream, 
		mut action_receiver: mpsc::Receiver<Arc<ClientAction>>, 
		mut server_action: ServerActionSender, 
		mut world_action: WorldActionSender,
		mut chunk_action: ChunkActionSender) -> Result<(), Box<dyn Error>> {
		
		let (reader, writer) = tokio::io::split(socket);
		
		// Broadcast thread
		tokio::spawn(async move {
			let mut packet_writer = FramedWrite::new(writer, PacketCodec::default());
			loop {
				if let Some(action) = action_receiver.recv().await {
					use ClientAction::*;
					debug!("Sending ClientAction: {:?}", action);
					let result = match &*action {
						SendPacket(packet) => packet_writer.send(&packet).await,
						SetChunkHandler(handle) => {
							chunk_action = handle.clone();
							Ok(())
						},
					};
					if let Err(_) = result {
						error!("Error with parsing ClientAction, Disconnecting"); break;
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
						EssentialTilesRequest(x, y) => {
							self.send_packet(Packet::Status(15, NetworkText::new("LegacyInterface.44"), 0)).await?;
							// Request cached WorldInfo data from world
							world_action.send(WorldAction::RequestChunkHandle(self.action.clone(), self.player.position) ).await?;
						}
						_ => warn!("Unimplemented Packet"), 
					}
				}else{ continue; }
			} else {
				error!("Error with reading packet: {:?}", result);
				break;
			}
		}
		Ok(())
	}
}

#[derive(Debug)]
pub enum ServerAction {
	ConnectClient(ClientActionSender), // Connect client thread to server action thread
	DisconnectClient(ClientActionSender), //
	Broadcast(Packet), // Broadcast to all clients
	
	Chat(String),
}
pub type ServerActionSender = mpsc::Sender<ServerAction>;
pub struct Server {
	clients: Vec<ClientActionSender>, // Channels to tell clients to send data
	
	addr: String, // Addr to host server on
}
impl Server {
	pub fn new(addr: &str) -> Self {
		Server {
			clients: Vec::with_capacity(8), // Typical, small server size
			addr: addr.into(),
		}
	}
	async fn handle(&mut self, mut action_receiver: mpsc::Receiver<ServerAction>, mut world_action: WorldActionSender) -> Result<(), Box<dyn Error>> {
		loop {
			let action = action_receiver.recv().await.unwrap();
			
			use ServerAction::*;
			use std::convert::TryInto;
			match action {
				ConnectClient(chan) => {
					self.clients.push(chan);
				},
				Broadcast(packet) => {
					let subaction = ClientAction::SendPacket(packet);
					let arc = Arc::new(subaction);
					for index in 0..self.clients.len() {
						if let Err(_) = self.clients[index as usize].send( arc.clone() ).await {
							self.clients.remove(index.try_into().unwrap());
						}
					}
				},
				Chat(s) => info!("Received Chat {}", s),
				//_ => warn!("Unimplemented Action")
			}
		}
	}
	
	pub async fn serve(server: Arc<Mutex<Server>>, world: Arc<Mutex<World>>) -> Result<(), Box<dyn Error>> {
		let server_lock = server.lock().await;
		let mut listener = TcpListener::bind(&server_lock.addr).await?;
		info!("Starting Terraria Server on {}", &server_lock.addr);
		
		// Spawns thread(s) that deal with world management functions
		let (world_action, world_action_receiver) = mpsc::channel(100);
		
		let world_clone = world.clone();
		tokio::spawn(async move {
			let mut lock = world_clone.lock().await;
			let result = lock.handle(world_action_receiver).await;
			match result {
				Err(err) => error!("World Thread Exited with error: {:?}", err),
				Ok(_) => info!("World thread exited normally"),
			}
		});
		
		// Action handler that listens on channel (so players can update world)
		let (server_action, server_receiver) = mpsc::channel(100);
		let w_tx = world_action.clone();
		
		let server_handle = server.clone();
		tokio::spawn(async move {
			let mut lock = server_handle.lock().await;
			let result = lock.handle(server_receiver, w_tx).await;
			match result {
				Err(err) => error!("Server Thread Exited with error: {:?}", err),
				Ok(_) => info!("Server Thread Exited Normally"),
			}
		});
		
		loop {
			let (socket, _) = listener.accept().await?; // Wait for new connection (or return Err)
			
			let mut sa_tx_copy = server_action.clone();
			let w_tx_copy = world_action.clone();
			
			let (mut client, action_receiver) = Client::new();
			if let Err(err) = sa_tx_copy.send(ServerAction::ConnectClient(client.action.clone())).await {
				error!("Server Closed: {:?}", err.to_string());
				break;
			}
			
			tokio::spawn(async move {
				info!(target: "client_thread", "New Client Connected {:?}", socket);
				let result = client.handle(socket, action_receiver, sa_tx_copy, w_tx_copy).await;
				match result {
					Err(error) => warn!("Client Error: {}", error),
					Ok(_) => info!("Client Disconnected"),
				}
			});
		}
		Ok(())
	}
}