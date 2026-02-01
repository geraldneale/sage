//! Blink Mojo puzzle definitions and loaders
//! 
//! This module contains the compiled CLVM puzzles for the Blink Mojo protocol.

use clvmr::{Allocator, NodePtr};
use clvmr::serde::node_from_bytes;

/// Compiled puzzle hex strings
const NEEDS_PRIVACY_HEX: &str = include_str!("puzzles/needs_privacy.clvm.hex");
const DECOY_VALUE_HEX: &str = include_str!("puzzles/decoy_value.clvm.hex");
const SOURCE_HEX: &str = include_str!("puzzles/source.clvm.hex");
const DECOY_HEX: &str = include_str!("puzzles/decoy.clvm.hex");

/// Container for all Blink Mojo puzzles
#[derive(Debug, Clone)]
pub struct BlinkPuzzles {
    pub needs_privacy: Vec<u8>,
    pub decoy_value: Vec<u8>,
    pub source: Vec<u8>,
    pub decoy: Vec<u8>,
}

impl BlinkPuzzles {
    /// Load all puzzles from embedded hex strings
    pub fn load() -> Result<Self, hex::FromHexError> {
        Ok(Self {
            needs_privacy: hex::decode(NEEDS_PRIVACY_HEX.trim())?,
            decoy_value: hex::decode(DECOY_VALUE_HEX.trim())?,
            source: hex::decode(SOURCE_HEX.trim())?,
            decoy: hex::decode(DECOY_HEX.trim())?,
        })
    }
    
    /// Deserialize a puzzle into CLVM NodePtr
    pub fn deserialize_puzzle(
        allocator: &mut Allocator,
        puzzle_bytes: &[u8],
    ) -> Result<NodePtr, std::io::Error> {
        node_from_bytes(allocator, puzzle_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_puzzles() {
        let puzzles = BlinkPuzzles::load().expect("Failed to load puzzles");
        assert!(!puzzles.needs_privacy.is_empty());
        assert!(!puzzles.decoy_value.is_empty());
        assert!(!puzzles.source.is_empty());
        assert!(!puzzles.decoy.is_empty());
    }
}
