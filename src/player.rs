#![allow(dead_code)]
extern crate bitflags;

bitflags! {
	struct Difficulty: u8 {
		const Softcore 		= 0b00000000 // 0
		const Mediumcore 	= 0b00000001 // 1
		const Harcore 		= 0b00000010 // 2
		const ExtraAccessory= 0b00000100 // 4
		const Creative 		= 0b00001000 // 8
	}
}
bitflags! {
	struct TorchState: u8 {
		UsingBiomeTorches = 0b00000001; // 1
		HappyFunTorchTime = 0b00000010; // 2
	}
}
struct Color {
	r: u8, g: u8, b: u8,
}
struct Inventory {

}
struct Appearance {
	skin: u8,
	hair: u8,
	hair_dye: u8,
	hide_visuals_1: u8,
	hide_visuals_2: u8,
	hide_misc: u8,
	hair_color: Color,
	skin_color: Color,
	eye_color: Color,
	shirt_color: Color,
	under_shift_color: Color,
	pants_color: Color,
	shoe_color: Color,
}

struct Player {
	id: u8,
	name: String,
	
	inventory: Inventory,
	appearance: Appearance,

	difficulty: Difficulty,
	torch_state: TorchState,
}