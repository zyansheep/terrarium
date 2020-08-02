use serde::{Deserialize, Serialize};

/// A stack of an item.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ItemStack {
    /// The size of the stack.
    pub stack: u16,
    /// The ID of the item.
    pub id: u16,
    /// The prefix/modifier of the item.
    pub prefix: u8,
}

/// A chest.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Chest {
    /// The name of the chest.
    pub name: String,
    /// The X tile coordinate of the chest.
    pub x: u32,
    /// The Y tile coordinate of the chest.
    pub y: u32,
    /// The items of the chest.
    pub items: Vec<ItemStack>,
}
