#![allow(dead_code)]
#![allow(unused_imports)]

use std::error::Error;
use std::sync::Arc;
use log::{info, trace, warn};

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_util::codec::{FramedRead, FramedWrite};
use tokio::stream::{self, StreamExt};
use futures::sink::SinkExt;

use crate::packet::{Packet, PacketCodec, PacketError};
use crate::world::World;
use crate::player::Player;

#[derive(Debug)]
enum ClientAction {
	SendPacket(Packet),
	ReceiveChat { chat: String },
}
struct Client {
	id: u8, // What the client thinks its index is
	player: Box<Player>,
	action: mpsc::Sender<Arc<ClientAction>>,
}
impl Client {
	fn new(action: mpsc::Sender<Arc<ClientAction>>) -> Self {
		Client {
			id: 0,
			player: Box::new(Player::default()),
			action,
		}
	}
	async fn send_packet(&mut self, packet: Packet) -> Result< (), mpsc::error::SendError<Arc<ClientAction>> > {
		self.action.send( Arc::new(ClientAction::SendPacket(packet)) ).await
	}
	async fn handle_new(socket: TcpStream, mut action_sender: mpsc::Sender<ServerAction>) -> Result<(), Box<dyn Error>> {
		info!(target: "client_handle", "New Client Connected {:?}", socket);

		
		let (reader, writer) = tokio::io::split(socket);
		
		// Send clientaction chanel to server
		let (action, mut action_receiver) = mpsc::channel(100);
		let _ = action_sender.send(ServerAction::Connect(action.clone())).await;
		
		let mut client = Client::new(action.clone());
		
		// Broadcast thread

		tokio::spawn(async move {
			let mut packet_writer = FramedWrite::new(writer, PacketCodec::default());
			loop {
				if let Some(action) = action_receiver.recv().await {
					use ClientAction::*;
					println!("Sending ClientAction: {:?}", action);
					let result = match &*action {
						SendPacket(packet) => packet_writer.send(&packet).await,
						//ReceiveChat{ ref chat } => println!("Sending Chat: \"{}\"", chat),
						_ => Ok(info!("Unimplemented ClientAction Received: {:?}", action)),
					};
					if let Err(_) = result {
						warn!("Error with sending packet, Disconnecting"); break;
					}
				} else {
					break;
				}
			}
		});
		
		client.player.name = "Uninitialized".into();
		// Reader thread
		let mut packet_reader = FramedRead::new(reader, PacketCodec::default());
		loop {
			let result = packet_reader.try_next().await;
			if let Ok(packet_or_none) = result {
				if let Some(packet) = packet_or_none {
					println!("Read Packet: {:?}", packet);
					match packet {
						Packet::ConnectRequest(s) => {
							if s == "Terraria230"{
								client.send_packet(Packet::SetUserSlot(0)).await? // Every client is always in user slot 0 (other players are dynamically set up to 256 user slots)
							} else {
								// Send disconnect packet and drop connection
							}
						},
						_ => warn!("Unimplemented Packet"), 
					}
				}else{ continue; }
			} else { info!("Error with reading packet: Disconnecting"); break; } //Else, stream closed
		}
		Ok(())
	}
}

enum ServerAction {
	Connect(mpsc::Sender<Arc<ClientAction>>), // Connect client thread to server action thread
	SendToClient(u8, Packet), // Send to specific client
	Broadcast(Packet), // Broadcast to all clients
	
	Chat(String),
}
pub struct Server {
	clients: Vec<mpsc::Sender<Arc<ClientAction>>>, // Channels to tell clients to send data
	action: mpsc::Sender<ServerAction>,
	action_receiver: mpsc::Receiver<ServerAction>,
	world: World, // World Data Here
	addr: String, // Addr to host server on
}
impl Server {
	pub fn new(world: World, addr: &str) -> Self {
		let (tx, rx) = mpsc::channel(100);
		Server {
			clients: Vec::with_capacity(8), // Typical, small server size
			action: tx,
			action_receiver: rx,
			world: world,
			addr: addr.into(),
		}
	}
	async fn action_handler(&mut self) {
		loop {
			let action = self.action_receiver.recv().await.unwrap();
			
			use ServerAction::*;
			use std::convert::TryInto;
			match action {
				Connect(chan) => {
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
				}
				Chat(s) => println!("Received Chat {}", s),
				//_ => println!("Unimplemented Action")
			}
		}
	}
	
	pub async fn start(mut self) -> Result<(), Box<dyn Error>> {
		let mut listener = TcpListener::bind(&self.addr).await?;
		println!("Starting Terraria Server on {}", &self.addr);
		
		// Channel to send actions from client thread to server thread
		
		let sa_channel = self.action.clone();
		// Action handler that listens on channel (so players can update world)
		tokio::spawn(async move {
			let _ = self.action_handler().await;
		});
		
		loop {
			let (socket, _) = listener.accept().await?; // Wait for new connection (or return Err)
			
			let sa_tx_copy = sa_channel.clone();
			tokio::spawn(async move {
				// Create new client object 
				// Initialize Reader and Sender thread
				let _ = Client::handle_new(socket, sa_tx_copy).await; // TODO: Log client errors...
			});
		}
	}
}