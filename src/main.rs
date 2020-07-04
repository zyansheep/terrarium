#[macro_use]
extern crate clap;
use clap::App;

#[macro_use]
extern crate bitflags;
extern crate tokio;
extern crate byteorder;
extern crate variant_encoding;

// Config loading & saving
mod config;
use config::Config;

// World module
mod world;
use world::World;

mod player;
mod server;
use server::Server;

#[tokio::main]
async fn main() {
	let yaml_args = load_yaml!("app.yml");
	let matches = App::from_yaml(yaml_args).get_matches();

	use std::path::Path;

	let mut config = Config::new(7777, "world.wld"); // Default port 7777, default world file name "world.wld" (in CWD)

	// if config file path passed, use that
	let mut config_path = Path::new("config.yml"); // Otherwise, use config.yml in current directory if exists
	if let Some(config_arg) = matches.value_of("config") {
		config_path = Path::new(config_arg);
		config = Config::from_file(config_path).expect("Could not parse yml file passed");
	} else {
		if config_path.exists() {
			config = Config::from_file(config_path)
				.expect("Could not parse config.yml file in current directory?");
		}
	}
	// Override config if different world file provided
	if let Some(world) = matches.value_of("world") {
		config.world = world.to_owned();
	}
	if let Some(port_str) = matches.value_of("port") {
		config.port = port_str.parse().expect("Error, port value not correct");
	}

	//println!("{:#?}", config);

	// Read world file
	let world = World::read_from_file(&config.world).expect("Could not parse world");
	//println!("{:?}", world);
	// Write world file
	//world.write_to_file("write_test.wld").unwrap();

	let address = "127.0.0.1:7777";
	let server = Server::new(world, address);
	
	server.start().await.unwrap(); // Run Server
}
