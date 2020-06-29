#[macro_use]
extern crate clap;
use clap::App;

use std::path::Path;

fn main() {
	let yaml = load_yaml!("cli.yml");
	let matches = App::from_yaml(yaml).get_matches();
		
	// Figure out where world file and config files are
	let mut world_path = Path::new("world.wld");
	if let Some(world) = matches.value_of("world") {
		world_path = Path::new(world);
	} else if !world_path.is_file() {
		panic!("World file is not file: {}", world_path.display());
	}
	//let world = World::new();
	//world.load(world_path).unwrap(); //This should call path.exists() and return error accordingly

	let mut port: u16 = 7777;
	if let Some(port_str) = matches.value_of("port") {
		port = port_str.parse().expect("Error, port value not correct");
	}

	//let server = Server::new();
	//server.start(port, world, etc...)
}
