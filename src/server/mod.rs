#![allow(dead_code)]
#![allow(unused_imports)]

use log::{trace, debug, info, warn, error};
use std::error::Error;
use std::sync::Arc;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio::stream::{self, StreamExt};
use tokio_util::codec::{FramedRead, FramedWrite};
use futures::sink::SinkExt;

pub mod cache;
//use cache::{Cache, CacheCommand};

use crate::packet::{Packet, PacketCodec, PacketError, types::NetworkText};
use crate::world::World;
use crate::player::Player;


#[derive(Debug)]
pub enum ClientAction {
	SendPacket(Packet),
	SendWorldInfo(Arc<RwLock<cache::WorldInfo>>), 
}
pub type ClientActionSender = mpsc::Sender<Arc<ClientAction>>;
struct Client {
	id: u8, // What the client thinks its index is
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
	async fn handle(&mut self, socket: TcpStream, mut server_action: mpsc::Sender<ServerAction>, mut action_receiver: mpsc::Receiver<Arc<ClientAction>>) -> Result<(), Box<dyn Error>> {
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
						//ReceiveChat{ ref chat } => println!("Sending Chat: \"{}\"", chat),
						SendWorldInfo(info_rwlock) => {
							let info = &*info_rwlock.read().await; // Await for read lock
							packet_writer.send(&Packet::WorldInfo(info.clone())).await // If aquired read lock, send to packet writer
						}
						//_ => Ok(warn!("Unimplemented ClientAction Received: {:?}", action)),
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
						WorldDataRequest => server_action.send(ServerAction::GetWorldData(self.action.clone())).await?,
						EssentialTilesRequest(x, y) => {
							self.send_packet(Packet::Status(15, NetworkText::new("LegacyInterface.44"), 0)).await?;
							//self.send_packet(Packet::Chunk())
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
	SendToClient(u8, Packet), // Send to specific client
	Broadcast(Packet), // Broadcast to all clients
	
	GetWorldData(ClientActionSender),
	Chat(String),
}
pub struct Server {
	clients: Vec<ClientActionSender>, // Channels to tell clients to send data
	action: mpsc::Sender<ServerAction>,
	action_receiver: mpsc::Receiver<ServerAction>,
	world: Arc<World>, // World Data Here
	addr: String, // Addr to host server on
	//cache_updater: mpsc::Sender<CacheCommand>,
}
impl Server {
	pub fn new(world: Arc<World>, addr: &str) -> Self {
		let (tx, rx) = mpsc::channel(100);
		//let (cache, updater) = Cache::new(world.clone());
		Server {
			clients: Vec::with_capacity(8), // Typical, small server size
			action: tx,
			action_receiver: rx,
			world: world,
			addr: addr.into(),
			//cache_updater: updater
		}
	}
	async fn action_handler(&mut self) -> Result<(), Box<dyn Error>> {
		//self.cache_updater.send(cache::CacheCommand::UpdateWorldInfo).await?;
		
		loop {
			let action = self.action_receiver.recv().await.unwrap();
			
			use ServerAction::*;
			use std::convert::TryInto;
			match action {
				ConnectClient(chan) => {
					self.clients.push(chan);
				},
				SendToClient(index, packet) => {
					if let Err(_) = self.clients[index as usize].send(Arc::new(ClientAction::SendPacket(packet))).await {
						self.clients.remove(index.try_into().unwrap());
					}
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
				GetWorldData(mut action_sender) => {
					info!("Received GetWorldData Command");
					let packet = Packet::WorldInfo( cache::WorldInfo::new(self.world.clone())? );
					info!("GetWorldData: {:?}", packet);
					let _ = action_sender.send( Arc::new(ClientAction::SendPacket(packet)) ).await;
				},
				Chat(s) => info!("Received Chat {}", s),
				//_ => warn!("Unimplemented Action")
			}
		}
	}
	
	pub async fn start(mut self) -> Result<(), Box<dyn Error>> {
		let mut listener = TcpListener::bind(&self.addr).await?;
		info!("Starting Terraria Server on {}", &self.addr);
		
		// Channel to send actions from client thread to server thread
		let mut sa_channel = self.action.clone();
		// Action handler that listens on channel (so players can update world)
		tokio::spawn(async move {
			let result = self.action_handler().await;
			match result {
				Err(err) => error!("Server Thread Exited with error: {:?}", err),
				Ok(_) => info!("Server Thread Exited Normally"),
			}
		});
		
		loop {
			let (socket, _) = listener.accept().await?; // Wait for new connection (or return Err)
			
			let sa_tx_copy = sa_channel.clone();
			
			let (mut client, action_receiver) = Client::new();
			if let Err(err) = sa_channel.send(ServerAction::ConnectClient(client.action.clone())).await {
				error!("Server Closed: {:?}", err.to_string());
				break;
			}
			
			tokio::spawn(async move {
				// Create new client object 
				// Initialize Reader and Sender thread
				info!(target: "client_thread", "New Client Connected {:?}", socket);
				let result = client.handle(socket, sa_tx_copy, action_receiver).await; // TODO: Log client errors...
				match result {
					Err(error) => warn!("Client Error: {}", error),
					Ok(_) => info!("Client Disconnected"),
				}
			});
		}
		Ok(())
	}
}