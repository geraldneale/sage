//! Blink Mojo settlement - building the spend bundle

use crate::blink::{BlinkMix, BlinkPuzzles};
use chia::protocol::{Coin, CoinSpend, SpendBundle};
use chia::bls::{sign, aggregate, SecretKey, Signature};
use clvmr::{Allocator, NodePtr};
use clvmr::serde::node_to_bytes;
use anyhow::Result;

const GENESIS_CHALLENGE_TESTNET10: [u8; 32] = hex_literal::hex!("ae83525ba8d1dd3f09b277de18ca3e43fc0af20d20c4b3e92ef2a48bd291ccb2");

pub struct BlinkSettlement {
    mix: BlinkMix,
    puzzles: BlinkPuzzles,
    
    faucet_sk: SecretKey,
    needs_privacy_sk: SecretKey,
    decoy_sk: SecretKey,
    decoy_value_sk: SecretKey,
    
    faucet_msg: Vec<u8>,
    needs_privacy_msg: Vec<u8>,
    decoy_msg: Vec<u8>,
    decoy_value_msg: Vec<u8>,
}

impl BlinkSettlement {
    pub fn new(
        mix: BlinkMix,
        faucet_sk: SecretKey,
        faucet_msg: Vec<u8>,
        needs_privacy_sk: SecretKey,
        needs_privacy_msg: Vec<u8>,
        decoy_sk: SecretKey,
        decoy_msg: Vec<u8>,
        decoy_value_sk: SecretKey,
        decoy_value_msg: Vec<u8>,
    ) -> Result<Self> {
        mix.validate().map_err(|e| anyhow::anyhow!(e))?;
        let puzzles = BlinkPuzzles::load()?;
        
        Ok(Self {
            mix,
            puzzles,
            faucet_sk,
            needs_privacy_sk,
            decoy_sk,
            decoy_value_sk,
            faucet_msg,
            needs_privacy_msg,
            decoy_msg,
            decoy_value_msg,
        })
    }
    
    pub fn build_spend_bundle(&self) -> Result<SpendBundle> {
        let mut allocator = Allocator::new();
        
        let faucet_spend = self.create_faucet_spend(&mut allocator)?;
        let needs_privacy_spend = self.create_needs_privacy_spend(&mut allocator)?;
        let decoy_spend = self.create_decoy_spend(&mut allocator)?;
        let decoy_value_spend = self.create_decoy_value_spend(&mut allocator)?;
        
        let sig1 = self.sign_coin(&self.faucet_sk, &self.faucet_msg, &self.mix.faucet_coin);
        let sig2 = self.sign_coin(&self.needs_privacy_sk, &self.needs_privacy_msg, &self.mix.needs_privacy_coin);
        let sig3 = self.sign_coin(&self.decoy_sk, &self.decoy_msg, &self.mix.decoy_coin);
        let sig4 = self.sign_coin(&self.decoy_value_sk, &self.decoy_value_msg, &self.mix.decoy_value_coin);
        
        let aggregated_sig = aggregate(&[sig1, sig2, sig3, sig4]);
        
        let spend_bundle = SpendBundle::new(
            vec![faucet_spend, needs_privacy_spend, decoy_spend, decoy_value_spend],
            aggregated_sig,
        );
        
        Ok(spend_bundle)
    }
    
    fn sign_coin(&self, sk: &SecretKey, msg: &[u8], coin: &Coin) -> Signature {
        let mut sig_data = msg.to_vec();
        sig_data.extend_from_slice(&coin.coin_id());
        sig_data.extend_from_slice(&GENESIS_CHALLENGE_TESTNET10);
        sign(sk, &sig_data)
    }
    
    fn curry_puzzle(&self, allocator: &mut Allocator, puzzle: &[u8], _args: Vec<NodePtr>) -> Result<NodePtr> {
        // TODO: Implement currying
        Ok(BlinkPuzzles::deserialize_puzzle(allocator, puzzle)?)
    }
    
    fn create_faucet_spend(&self, allocator: &mut Allocator) -> Result<CoinSpend> {
        let puzzle_reveal = self.curry_puzzle(allocator, &self.puzzles.faucet, vec![])?;
        let solution = allocator.nil();
        
        Ok(CoinSpend::new(
            self.mix.faucet_coin.clone(),
            node_to_bytes(allocator, puzzle_reveal)?.into(),
            node_to_bytes(allocator, solution)?.into(),
        ))
    }
    
    fn create_needs_privacy_spend(&self, allocator: &mut Allocator) -> Result<CoinSpend> {
        let puzzle_reveal = self.curry_puzzle(allocator, &self.puzzles.needs_privacy, vec![])?;
        let solution = allocator.nil();
        
        Ok(CoinSpend::new(
            self.mix.needs_privacy_coin.clone(),
            node_to_bytes(allocator, puzzle_reveal)?.into(),
            node_to_bytes(allocator, solution)?.into(),
        ))
    }
    
    fn create_decoy_spend(&self, allocator: &mut Allocator) -> Result<CoinSpend> {
        let puzzle_reveal = self.curry_puzzle(allocator, &self.puzzles.decoy, vec![])?;
        let solution = allocator.nil();
        
        Ok(CoinSpend::new(
            self.mix.decoy_coin.clone(),
            node_to_bytes(allocator, puzzle_reveal)?.into(),
            node_to_bytes(allocator, solution)?.into(),
        ))
    }
    
    fn create_decoy_value_spend(&self, allocator: &mut Allocator) -> Result<CoinSpend> {
        let puzzle_reveal = self.curry_puzzle(allocator, &self.puzzles.decoy_value, vec![])?;
        let solution = allocator.nil();
        
        Ok(CoinSpend::new(
            self.mix.decoy_value_coin.clone(),
            node_to_bytes(allocator, puzzle_reveal)?.into(),
            node_to_bytes(allocator, solution)?.into(),
        ))
    }
}
