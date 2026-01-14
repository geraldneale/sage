//! Blink Mojo mixing transaction structures

use chia::protocol::Coin;

/// A complete Blink Mojo mixing transaction
#[derive(Debug, Clone)]
pub struct BlinkMix {
    // The faucet coin whose parent_id we hijack
    pub faucet_coin: Coin,
    pub faucet_parent_id: [u8; 32],
    
    // Real coin needing privacy
    pub needs_privacy_coin: Coin,
    pub needs_privacy_value: u64,
    pub needs_privacy_destination: [u8; 32],  // puzzle hash where it goes
    
    // Small decoy from your wallet
    pub decoy_coin: Coin,
    
    // Decoy value output (>= needs_privacy, back to your wallet)
    pub decoy_value_amount: u64,
    pub decoy_value_destination: [u8; 32],  // back to your wallet
}

impl BlinkMix {
    /// Validate that the mix parameters are correct for privacy
    pub fn validate(&self) -> Result<(), String> {
        // Critical: decoy_value must be >= needs_privacy
        if self.decoy_value_amount < self.needs_privacy_value {
            return Err(format!(
                "Privacy violation: decoy_value ({}) must be >= needs_privacy ({})",
                self.decoy_value_amount, self.needs_privacy_value
            ));
        }
        Ok(())
    }
}
