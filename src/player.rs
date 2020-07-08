#![allow(dead_code)]
#![allow(non_upper_case_globals)]

#![allow(unused_imports)]

use std::io;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use variant_encoding::{VarStringReader, VarStringWriter};

#[derive(Error, Debug)]
pub enum PlayerError {
	#[error("Error Reading / Writing Packet Data")]
	ReadError(#[from] io::Error),
	#[error("Packet received that wrote to unmodifiable field: {0}")]
	WrongField(&'static str),
}

bitflags! {
	#[derive(Default)]
	pub struct Difficulty: u8 {
		const Softcore 		= 0b00000000; // 0
		const Mediumcore 	= 0b00000001; // 1
		const Harcore 		= 0b00000010; // 2
		const ExtraAccessory= 0b00000100; // 4
		const Creative 		= 0b00001000; // 8
	}
}
bitflags! {
	#[derive(Default)]
	pub struct TorchState: u8 {
		const UsingBiomeTorches = 0b00000001; // 1
		const HappyFunTorchTime = 0b00000010; // 2
	}
}
#[derive(Default, Debug, PartialEq)]
pub struct Color {
	r: u8, g: u8, b: u8,
}
impl Color {
	pub fn read(reader: &mut impl io::BufRead) -> Result<Color, io::Error> {
		Ok(Color {
			r: reader.read_u8()?,
			g: reader.read_u8()?,
			b: reader.read_u8()?,
		})
	}
}
#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct Item {
	id: u16,
	amount: u16,
	prefix: u8,
}
#[derive(Debug, PartialEq)]
pub struct Inventory {
	//TODO: can't have arrays over 32 items in rust stable, figure out a workaround that doesn't use Vec<>
	/*inventory: [Item; 58], // 40 Inventory Slots, 10 Hotbar Slots, 4 Ammo, 4 Coin (index = 0..58)
	trash: Item, // Trash slot is down here for some reason... (index = 179)
	
	armor: [Item; 20], // 3 Armor, 7 Accessories, 10 Vanity (index = 59..78)
	dye: [Item; 10], // 10 Dye (index = 79..88)
	
	misc_equips: [Item; 5], // Pets, Minecart, Light, Grappling Hook, etc. (index = 89..93)
	misc_dyes: [Item; 5], // Dyes for above (index = 94..98)
	
	piggy_bank: [Item; 40], // (index = 99..138)
	safe: [Item; 40], // (index = 139..178)
	defenders_forge: [Item; 40], // (index = 180..219)
	void_vault: [Item; 40], // (index = 220..259)*/
	
	inventory: Vec<Item>, // 40 Inventory Slots, 10 Hotbar Slots, 4 Ammo, 4 Coin (index = 0..58)
	trash: Item, // Trash slot is down here for some reason... (index = 179)
	
	armor: Vec<Item>, // 3 Armor, 7 Accessories, 10 Vanity (index = 59..78)
	dye: Vec<Item>, // 10 Dye (index = 79..88)
	
	misc_equips: Vec<Item>, // Pets, Minecart, Light, Grappling Hook, etc. (index = 89..93)
	misc_dyes: Vec<Item>, // Dyes for above (index = 94..98)
	
	piggy_bank: Vec<Item>, // (index = 99..138)
	safe: Vec<Item>, // (index = 139..178)
	defenders_forge: Vec<Item>, // (index = 180..219)
	void_vault: Vec<Item>, // (index = 220..259)
}
impl Default for Inventory {
	fn default() -> Self {
		Self {
			inventory: [Item::default(); 59].to_vec(), // 40 Inventory Slots, 10 Hotbar Slots, 4 Ammo, 4 Coin (index = 0..58) and 1 other slot?
			trash: Item::default(), // Trash slot is down here for some reason... (index = 179)
			
			armor: [Item::default(); 20].to_vec(), // 3 Armor, 7 Accessories, 10 Vanity (index = 59..78)
			dye: [Item::default(); 10].to_vec(), // 10 Dye (index = 79..88)
			
			misc_equips: [Item::default(); 5].to_vec(), // Pets, Minecart, Light, Grappling Hook, etc. (index = 89..93)
			misc_dyes: [Item::default(); 5].to_vec(), // Dyes for above (index = 94..98)
			
			piggy_bank: [Item::default(); 40].to_vec(), // (index = 99..138)
			safe: [Item::default(); 40].to_vec(), // (index = 139..178)
			defenders_forge: [Item::default(); 40].to_vec(), // (index = 180..219)
			void_vault: [Item::default(); 40].to_vec(), // (index = 220..259)
		}
	}
}
impl Inventory {
	pub fn update_slot(&mut self, packet: Packet) -> Result<(), PlayerError> {
		if let Packet::PlayerInventorySlot{slot_index, amount, item_prefix, net_id} = packet {
			// TODO: Make sure items sent can actually be stored in character slots
			let index = slot_index as usize;
			let item = Item{id: net_id, amount: amount, prefix: item_prefix};
			match index {
				0..=58 => self.inventory[index] = item,
				179 => self.trash = item,
				59..=78 => self.armor[index - 59] = item,
				79..=88 => self.dye[index - 79] = item,
				89..=93 => self.misc_equips[index - 89] = item,
				94..=98 => self.misc_dyes[index - 94] = item,
				
				99..=138 => self.piggy_bank[index - 99] = item,
				139..=178 => self.safe[index - 139] = item,
				180..=219 => self.defenders_forge[index - 180] = item,
				220..=259 => self.void_vault[index - 220] = item,
				_ => return Err(PlayerError::WrongField("Invalid Slot Index"))
			}
		}
		Ok(())
	}
}

#[derive(Default, Debug, PartialEq)]
pub struct Appearance {
	pub name: String,
	
	pub skin: u8,
	pub hair: u8,
	pub hair_dye: u8,
	pub hide_visuals_1: u8,
	pub hide_visuals_2: u8,
	pub hide_misc: u8,
	pub hair_color: Color,
	pub skin_color: Color,
	pub eye_color: Color,
	pub shirt_color: Color,
	pub under_shift_color: Color,
	pub pants_color: Color,
	pub shoe_color: Color,
	
	pub unknown_trait: u8,
	pub unknown_trait2: u8,
}
impl Appearance {
	pub fn read(reader: &mut impl io::BufRead) -> Result<Appearance, crate::packet::PacketError> {
		let mut appearance = Appearance::default();
		appearance.skin = reader.read_u8()?;
		appearance.hair = reader.read_u8()?;
		
		appearance.name = reader.read_varstring()?;
		if appearance.name.is_empty() { return Err(crate::packet::PacketError::InvalidField) }
		
		appearance.hair_dye = reader.read_u8()?;
		appearance.hide_visuals_1 = reader.read_u8()?;
		appearance.hide_visuals_2 = reader.read_u8()?;
		appearance.hide_misc = reader.read_u8()?;
		appearance.hair_color = Color::read(reader)?;
		appearance.skin_color = Color::read(reader)?;
		appearance.eye_color = Color::read(reader)?;
		appearance.shirt_color = Color::read(reader)?;
		appearance.under_shift_color = Color::read(reader)?;
		appearance.pants_color = Color::read(reader)?;
		appearance.shoe_color = Color::read(reader)?;
		
		//appearance.unknown_color = Color::read(reader)?;
		appearance.unknown_trait = reader.read_u8()?;
		appearance.unknown_trait2 = reader.read_u8()?;
		
		Ok(appearance)
	}
	pub fn init(&mut self, other: Appearance) -> Result<(), PlayerError> { 
		if !self.name.is_empty() {
			Err(PlayerError::WrongField("Can't Modify Player Appearance"))
		} else {
			*self = other;
			println!("{:?}", self);
			Ok(())
		}
	}
}

#[derive(Default, Debug, PartialEq)]
pub struct Status {
	pub hp: u16,
	pub max_hp: u16,
	
	pub mana: u16,
	pub max_mana: u16,
	
	pub buffs: [u16; 22],
}
use crate::packet::Packet;
impl Status {
	pub fn init(&mut self, packet: Packet) -> Result<(), PlayerError> {
		match packet {
			Packet::PlayerHp{hp: _, max_hp} => {
				if self.max_hp == 0 {
					self.max_hp = max_hp;
					self.hp = self.max_hp;
					Ok(())
				} else { Err(PlayerError::WrongField("Can't Modify Hp")) }
			},
			Packet::PlayerMana{mana: _, max_mana} => {
				if self.max_mana == 0 {
					self.max_mana = max_mana;
					self.mana = self.max_mana;
					Ok(())
				} else { Err(PlayerError::WrongField("Can't Modify Mana")) }
			},
			Packet::PlayerBuff{buffs} => {
				if self.buffs == [0u16; 22] {
					self.buffs = buffs;
					Ok(())
				} else { Err(PlayerError::WrongField("Can't Modify Buffs")) }
			},
			_ => Err(PlayerError::WrongField("Unknown Status Packet")),
		}
	}
}

#[derive(Default, Debug, PartialEq)]
pub struct Player {
	pub id: u8,
	pub uuid: String, // uuid of the player TODO: what is this used for?
	
	pub status: Status, // Holds hp, mana, buffs etc.
	pub inventory: Inventory, // Whats in your inventory?
	pub appearance: Appearance, // Appearance information

	pub difficulty: Difficulty,
	pub torch_state: TorchState,
}