//! Blink Mojo mixing transaction structures

use chia::protocol::Coin;

#[derive(Debug, Clone)]
pub struct BlinkMix {
    // The faucet coin whose parent_id we hijack
    pub faucet_coin: Coin,
    pub faucet_parent_id: [u8; 32],
    
    // Real coin needing privacy
    pub needs_privacy_coin: Coin,
    pub needs_privacy_value: u64,
    pub needs_privacy_destination: [u8; 32],
    
    // Small decoy from your wallet
    pub decoy_coin: Coin,
    
    // Decoy value coin (actually gets spent)
    pub decoy_value_coin: Coin,
    pub decoy_value_amount: u64,
    pub decoy_value_destination: [u8; 32],
}

impl BlinkMix {
    pub fn validate(&self) -> Result<(), String> {
        if self.decoy_value_amount < self.needs_privacy_value {
            return Err(format!(
                "Privacy violation: decoy_value ({}) must be >= needs_privacy ({})",
                self.decoy_value_amount, self.needs_privacy_value
            ));
        }
        Ok(())
    }
}
