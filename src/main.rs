#[macro_use] extern crate clap;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate thiserror;
extern crate log;
extern crate env_logger;

use clap::App;
use log::LevelFilter;
use env_logger::Builder;
use terrarium_world::World;
use std::fs::File;
use log::{info};

// Config loading & saving

mod config;
use config::Config;

mod packet;
mod player;

mod errors;

mod server;
use server::Server;

#[tokio::main]
async fn main() {
	let mut builder = Builder::new();
    builder.filter_level(LevelFilter::Trace);
	builder.init();
	
	let yaml_args = load_yaml!("app.yml");
	let matches = App::from_yaml(yaml_args).get_matches();
	
	if let Some(matches) = matches.subcommand_matches("convert") {
		// TODO: progress bars
		let input_file = matches.value_of("input").expect("Please specify input file with --input or -i");
		let output_file = matches.value_of("output").expect("Please specify output file with --output or -o");
		
		let mut input = File::open(input_file).expect(&format!("Unable to read input file: {}", input_file)[..]);
		info!("Reading Vanilla World: {}", input_file);
		let world = World::read_vanilla(&mut input).expect("Failed to parse vanilla world");
		
		let mut output = File::create(output_file).expect(&format!("Unable to create output file: {}. Are the permissions wrong?", output_file));
		info!("Writing Terraria World: {}", output_file);	
		world.write(&mut output).expect("Failed to Output Wirkd");
		
		info!("Finished!");
		return ();
	}

	use std::path::Path;
	
	let mut config = Config::new("127.0.0.1", 7777, "world.twld"); // Default port 7777, default world file name "world.wld" (in CWD)

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
	let mut world_file = File::open(&config.world).expect("Could not find terrarium world file");

	info!("Loading World: {}", config.world);
	use std::sync::Arc;
	let world = Arc::new(World::read(&mut world_file).expect("Could not read world"));
	
	let server = Server::new(world.clone(), &config.get_address());
	
	// TODO: SIGINT/SIGTERM catching to gracefullly shutdown server (and save world)
	let _result = server.start().await; // Run Server
	
	world.write(&mut world_file).expect("Failed to save world");
}
