#![allow(unused_imports)]
#![allow(dead_code)]

use log::{trace, debug, info, warn, error};
use std::{
	error::Error,
	sync::Arc,
	collections::HashMap,
};

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock, Mutex};
use tokio::stream::{self, StreamExt};
use tokio_util::codec::{FramedRead, FramedWrite};
use futures::sink::SinkExt;

use crate::packet::{Packet, PacketCodec, PacketError, types::NetworkText};
use crate::world::*;

pub mod packet;

pub mod client;
pub mod player;

pub use client::{Client, ClientAction, ClientActionSender};

#[derive(Debug)]
pub enum ServerAction {
	ConnectClient(String, ClientActionSender), // Connect client thread to server action thread
	DisconnectClient(String), // Disconnect client
	Broadcast(Packet), // Broadcast to all clients
	
	Chat(String),
}
pub type ServerActionSender = mpsc::Sender<ServerAction>;
pub struct Server {
	clients: HashMap<usize, ClientActionSender>, // Channels to tell clients to send data
	names: HashMap<String, usize>,
	addr: String, // Addr server is hosting on
}
impl Server {
	pub fn new(addr: &str) -> Self {
		Server {
			clients: HashMap::with_capacity(8),
			names: HashMap::with_capacity(8),
			addr: addr.into(),
		}
	}
	async fn handle(&mut self, mut action_receiver: mpsc::Receiver<ServerAction>) -> Result<(), Box<dyn Error>> {
		loop {
			let action = action_receiver.recv().await.unwrap();
			
			use ServerAction::*;
			use std::convert::TryInto;
			match action {
				ConnectClient(name, mut chan) => {
					let id = self.names.len();
					chan.send(ClientAction::SetClientID(id)).await?;
					self.names.insert(name, id);
					self.clients.insert(id, chan);
				},
				DisconnectClient(name) => {
					if let Some(id) = self.names.remove(&name) {
						self.clients.remove(&id);
					} else {
						error!("Client {:?} allready left, but Disconnect ServerAction was called again", name);
					}
				},
				Chat(s) => info!("Received Chat {}", s),
				_ => warn!("Unimplemented Action")
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
		
		let server_handle = server.clone();
		tokio::spawn(async move {
			let mut lock = server_handle.lock().await;
			let result = lock.handle(server_receiver).await;
			match result {
				Err(err) => error!("Server Thread Exited with error: {:?}", err),
				Ok(_) => info!("Server Thread Exited Normally"),
			}
		});
		
		loop {
			let (socket, _) = listener.accept().await?; // Wait for new connection (or return Err)
			
			//let mut wld_tx_copy = world_action.clone();
			
			let (mut client, mut action_receiver) = Client::new();
			
			//wld_tx_copy.send(WorldAction::SpawnClient(client.action.clone(), None));
			
			let chunk_action = {
				let recv_action = action_receiver.recv().await;
				if let Some(action) = recv_action {
					if let ClientAction::UpdateChunkHandler(chunk_action) = action {
						Arc::new(Mutex::new(chunk_action))
					} else { 
						error!("Received invalid ClientAction: {:?}", action);
						continue;
					}
				} else { error!("All Senders dropped for client"); continue; }
			};
			
			let server_action = server_action.clone();
			let world_action = world_action.clone();
			tokio::spawn(async move {
				info!(target: "client_thread", "New Client Connected {:?}", socket);
				let result = client.handle(socket, action_receiver, server_action, world_action, chunk_action).await;
				match result {
					Err(error) => warn!("Client Error: {}", error),
					Ok(_) => info!("Client Disconnected"),
				}
			});
		}
		//Ok(())
	}
}