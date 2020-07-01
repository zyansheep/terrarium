#![allow(dead_code)]

extern crate serde;
extern crate serde_yaml;

use serde::{Serialize, Deserialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
	pub port: u16,
	pub world: String,
	#[serde(skip_serializing, skip_deserializing)]
	pub from_file: bool,
}

impl Config {
	pub fn new(port: u16, world: &str) -> Self {
		Config {
			port: port,
			world: world.to_owned(),
			from_file: false
		}
	}

	pub fn from_file(path: &Path) -> Result<Config, Box<dyn Error>> {
		let file = File::open(path)?;
		let mut tmp: Self = serde_yaml::from_reader(file)?;
		tmp.from_file = true;
		Ok(tmp)
	}
	
	pub fn to_file(&self, path: &Path) -> Result<(), Box<dyn Error>> {
		let file = File::create(path)?;
		serde_yaml::to_writer(file, self)?;
		Ok(())
	}
}
