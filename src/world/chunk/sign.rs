use serde::{Deserialize, Serialize};

/// A sign.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Sign {
    /// The X tile coordinate of the sign.
    pub x: u32,
    /// The Y tile coordinate of the sign.
    pub y: u32,
    /// The text of the sign.
    pub text: String,
}
