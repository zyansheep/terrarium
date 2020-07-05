#![allow(dead_code)]

use std::error::Error;
use std::collections::LinkedList;
use std::sync::Arc;
use std::io;

use tokio::io::{AsyncReadExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;


use crate::packet::Packet;
use crate::world::World;
use crate::player::Player;

enum ClientAction {
	RequestJoin(),
	ReceiveChat { chat: String },
}
struct Client {
	id: u8, // What the client thinks its index is
	player: Player,
}


impl Client {
	fn new() -> Self {
		Client {
			id: 0,
			player: Player::default(),
		}
	}
	async fn handle_new(socket: TcpStream, mut action_sender: mpsc::Sender<ServerAction>) -> Result<(), Box<dyn Error>> {
		let mut client = Client::new();
		
		let (mut reader, mut _writer) = tokio::io::split(socket);
		
		// Initialize broadcast channel and send to server
		let (broadcast_sender, mut broadcast_receiver) = mpsc::channel(100);
		let _ = action_sender.send(ServerAction::Connect(broadcast_sender)).await;
		
		// Broadcast thread
		tokio::spawn(async move {
			loop {
				if let Some(action) = broadcast_receiver.recv().await {
					use ClientAction::*;
					match &*action {
						ReceiveChat{ ref chat } => println!("Sending Chat: \"{}\"", chat),
						_ => println!("Unknown Action Received"),
					}
				} else {
					break;
				}
			}
		});
		
		client.player.name = "Uninitialized".into();
		// Reader thread
		let mut buf = [0u8; 1024];
		loop {
			reader.read(&mut buf).await?;
			//reader.read_buf(&mut buf).await?; // Read to temp buffer
			//reader.poll_read(&mut buf).await?;
			let mut bufr = io::BufReader::new(&buf[..]); //create tmp bufreader
			
			let pkt = Packet::read(&mut bufr).unwrap(); // Parse packet
			println!("{:?}", pkt);
			
			break;
		}
		Ok(())
	}
}

enum ServerAction {
	Connect(mpsc::Sender<Arc<ClientAction>>),
	Chat(String),
}
pub struct Server {
	clients: LinkedList<mpsc::Sender< Arc<ClientAction> >>, // Channels to tell clients to send data
	world: World, // World Data Here
	addr: String, // Addr to host server on
}
impl Server {
	pub fn new(world: World, addr: &str) -> Self {
		Server {
			clients: LinkedList::new(),
			world: world,
			addr: addr.into(),
		}
	}
	async fn action_handler(&mut self, mut receiver: mpsc::Receiver<ServerAction>) {
		let val = receiver.recv().await.unwrap();
		use ServerAction::*;
		match val {
			Connect(s) => self.clients.push_back(s),
			Chat(s) => println!("Received Chat {}", s),
			//_ => println!("Unimplemented Action")
		}
	}
	async fn broadcast(&mut self, action: ClientAction) -> Result<(), Box<dyn Error>> {
		let action_counter = Arc::new(action); // Shared resource (READ ONLY!!!)
		
		// Loop through all client channels, dropping any that Err
		
		// Using Nightly Feature std::collections::linked_list::Cursor;
		/*let mut cursor = self.clients.cursor_front_mut();
		loop {
			if let Some(chan) = cursor.peek_next() {
				if let Err(_) = chan.send(arc.clone()).await {
					cursor.remove_current();
				}
			} else {
				break;
			}
		}*/
		
		let mut new_clients = LinkedList::new();
		loop {
			if let Some(mut chan) = self.clients.pop_front() {
				if let Ok(_) = chan.send(action_counter.clone()).await {
					new_clients.push_back(chan);
				} // Else, channel sender dropped
			} else { break; }
		}
		self.clients = new_clients;
		
		Ok(())
	}

	pub async fn start(mut self) -> Result<(), Box<dyn Error>> {
		let mut listener = TcpListener::bind(&self.addr).await?;
		println!("Starting Terraria Server on {}", &self.addr);
		
		// Channel to send actions from client thread to server thread
		let (server_action_transmitter, server_action_receiver) = mpsc::channel(100);
		
		// Action handler that listens on channel (so players can update world)
		tokio::spawn(async move {
			let _ = self.action_handler(server_action_receiver).await;
		});
		
		loop {
			let (socket, _) = listener.accept().await?; // Wait for new connection (or return Err)
			
			let sa_tx_copy = server_action_transmitter.clone();
			tokio::spawn(async move {
				// Create new client object 
				// Initialize Reader and Sender thread
				let _ = Client::handle_new(socket, sa_tx_copy).await; // TODO: Log client errors...
			});
		}
	}
}