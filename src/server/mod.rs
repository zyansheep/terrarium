#![allow(unused_imports)]

use log::{trace, debug, info, warn, error};
use std::error::Error;
use std::sync::Arc;
use arc_swap::ArcSwap;

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
	async fn handle(&mut self, mut action_receiver: mpsc::Receiver<ServerAction>) -> Result<(), Box<dyn Error>> {
		loop {
			let action = action_receiver.recv().await.unwrap();
			
			use ServerAction::*;
			use std::convert::TryInto;
			match action {
				ConnectClient(chan) => {
					self.clients.push(chan);
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
		let w_tx = world_action.clone();
		
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
			
			let mut sa_tx_copy = server_action.clone();
			let mut wld_tx_copy = world_action.clone();
			
			let (mut client, mut action_receiver) = Client::new();
			if let Err(err) = sa_tx_copy.send(ServerAction::ConnectClient(client.action.clone())).await {
				error!("Server Closed: {:?}", err.to_string());
				break;
			}
			
			wld_tx_copy.send(WorldAction::SpawnClient(client.action.clone(), None));
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
			
			tokio::spawn(async move {
				info!(target: "client_thread", "New Client Connected {:?}", socket);
				let result = client.handle(socket, action_receiver, sa_tx_copy, wld_tx_copy, chunk_action).await;
				match result {
					Err(error) => warn!("Client Error: {}", error),
					Ok(_) => info!("Client Disconnected"),
				}
			});
		}
		Ok(())
	}
}