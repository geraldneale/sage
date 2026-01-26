//! Blink Mojo settlement - building the spend bundle

use crate::blink::{BlinkMix, BlinkPuzzles};
use chia::protocol::{Coin, CoinSpend, SpendBundle};
use chia::bls::{sign, aggregate, SecretKey, Signature, PublicKey};
use clvmr::{Allocator, NodePtr};
use clvmr::serde::node_to_bytes;
use anyhow::Result;
use chia::clvm_utils::CurriedProgram;
use chia::clvm_traits::ToClvm;

const GENESIS_CHALLENGE_TESTNET10: [u8; 32] = hex_literal::hex!("ae83525ba8d1dd3f09b277de18ca3e43fc0af20d20c4b3e92ef2a48bd291ccb2");

// Curry argument types
type FaucetArgs = ((PublicKey, Vec<u8>), (u64, [u8; 32]));
type NeedsPrivacyArgs = (PublicKey, Vec<u8>);
type DecoyArgs = ((PublicKey, Vec<u8>), (u64, [u8; 32]));
type DecoyValueArgs = (PublicKey, Vec<u8>);

#[derive(Debug)]
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
    
    // Helper to build curry args for 2 arguments: (c (q . arg1) (c (q . arg2) 1))
    // Helper to build curry args for 2 arguments: (c (q . arg1) (c (q . arg2) 1))
    // Helper to build curry args for 2 arguments: (c (q . arg1) (c (q . arg2) 1))
    // Specialized for (PublicKey, Vec<u8>)
    pub(crate) fn build_curry_args_2(
        allocator: &mut Allocator,
        pk: PublicKey,
        msg: Vec<u8>,
    ) -> Result<NodePtr> {
        let one = allocator.one();
        let four = allocator.new_small_number(4)?; // cons
        let nil = allocator.nil();
        
        // Convert PublicKey to atom
        let pk_atom = allocator.new_atom(&pk.to_bytes())?;
        let quoted_pk = allocator.new_pair(one, pk_atom)?;
        
        // Convert msg Vec<u8> to atom (NOT a list!)
        let msg_atom = allocator.new_atom(&msg)?;
        let quoted_msg = allocator.new_pair(one, msg_atom)?;
        
        // Build: (c (q . msg) 1)
        let step1 = allocator.new_pair(one, nil)?;
        let step2 = allocator.new_pair(quoted_msg, step1)?;
        let inner = allocator.new_pair(four, step2)?;
        
        // Build: (c (q . pk) inner)
        let step3 = allocator.new_pair(inner, nil)?;
        let step4 = allocator.new_pair(quoted_pk, step3)?;
        let args = allocator.new_pair(four, step4)?;
        
        Ok(args)
    }
    
    // Helper for 4 arguments: (PublicKey, Vec<u8>, u64, [u8; 32])
    // Helper for 4 arguments: (PublicKey, Vec<u8>, u64, [u8; 32])
    pub(crate) fn build_curry_args_4(
        allocator: &mut Allocator,
        pk: PublicKey,
        msg: Vec<u8>,
        amount: u64,
        puzzle_hash: [u8; 32],
    ) -> Result<NodePtr> {
        use chia::clvm_traits::ToClvm;
        
        let one = allocator.one();
        let four = allocator.new_small_number(4)?; // cons
        let nil = allocator.nil();
        
        // Convert to NodePtr - use ToClvm for u64 to get minimal encoding
        let pk_atom = allocator.new_atom(&pk.to_bytes())?;
        let msg_atom = allocator.new_atom(&msg)?;
        let amount_ptr = amount.to_clvm(allocator)?;  // Proper u64 encoding
        let ph_atom = allocator.new_atom(&puzzle_hash)?;
        
        // Quote them
        let quoted_pk = allocator.new_pair(one, pk_atom)?;
        let quoted_msg = allocator.new_pair(one, msg_atom)?;
        let quoted_amount = allocator.new_pair(one, amount_ptr)?;
        let quoted_ph = allocator.new_pair(one, ph_atom)?;
        
        // Build from inside out: (c (q . ph) 1)
        let step1 = allocator.new_pair(one, nil)?;
        let step2 = allocator.new_pair(quoted_ph, step1)?;
        let cons4 = allocator.new_pair(four, step2)?;
        
        // (c (q . amount) cons4)
        let step3 = allocator.new_pair(cons4, nil)?;
        let step4 = allocator.new_pair(quoted_amount, step3)?;
        let cons3 = allocator.new_pair(four, step4)?;
        
        // (c (q . msg) cons3)
        let step5 = allocator.new_pair(cons3, nil)?;
        let step6 = allocator.new_pair(quoted_msg, step5)?;
        let cons2 = allocator.new_pair(four, step6)?;
        
        // (c (q . pk) cons2)
        let step7 = allocator.new_pair(cons2, nil)?;
        let step8 = allocator.new_pair(quoted_pk, step7)?;
        let args = allocator.new_pair(four, step8)?;
        
        Ok(args)
    }
    
    fn curry_puzzle(
        &self,
        allocator: &mut Allocator,
        puzzle: &[u8],
        args: NodePtr,
    ) -> Result<NodePtr> {
        use chia::clvm_utils::CurriedProgram;
        use chia::clvm_traits::ToClvm;
        
        let puzzle_ptr = BlinkPuzzles::deserialize_puzzle(allocator, puzzle)?;
        
        let curried = CurriedProgram {
            program: puzzle_ptr,
            args,
        };
        
        curried.to_clvm(allocator).map_err(|e| anyhow::anyhow!("Curry error: {}", e))
    }
    
    fn create_faucet_spend(&self, allocator: &mut Allocator) -> Result<CoinSpend> {
        // Curry: (pk, msg, amount, anon_wallet_hash)
        let faucet_pk = self.faucet_sk.public_key();
        
        let args = Self::build_curry_args_4(
            allocator,
            faucet_pk,
            self.faucet_msg.clone(),
            self.mix.needs_privacy_value,
            self.mix.needs_privacy_destination,
        )?;
        
        let puzzle_reveal = self.curry_puzzle(allocator, &self.puzzles.faucet, args)?;
        let solution = allocator.nil();
        
        Ok(CoinSpend::new(
            self.mix.faucet_coin.clone(),
            node_to_bytes(allocator, puzzle_reveal)?.into(),
            node_to_bytes(allocator, solution)?.into(),
        ))
    }
    
    fn create_needs_privacy_spend(&self, allocator: &mut Allocator) -> Result<CoinSpend> {
        // Curry: (pk, msg)
        let needs_privacy_pk = self.needs_privacy_sk.public_key();
        
        let args = Self::build_curry_args_2(
            allocator,
            needs_privacy_pk,
            self.needs_privacy_msg.clone(),
        )?;
        
        let puzzle_reveal = self.curry_puzzle(allocator, &self.puzzles.needs_privacy, args)?;
        let solution = allocator.nil();
        
        Ok(CoinSpend::new(
            self.mix.needs_privacy_coin.clone(),
            node_to_bytes(allocator, puzzle_reveal)?.into(),
            node_to_bytes(allocator, solution)?.into(),
        ))
    }
    
    fn create_decoy_spend(&self, allocator: &mut Allocator) -> Result<CoinSpend> {
        // Curry: (pk, msg, amount, known_wallet_hash)
        let decoy_pk = self.decoy_sk.public_key();
        
        let args = Self::build_curry_args_4(
            allocator,
            decoy_pk,
            self.decoy_msg.clone(),
            self.mix.decoy_coin.amount,
            self.mix.needs_privacy_destination,
        )?;
        
        let puzzle_reveal = self.curry_puzzle(allocator, &self.puzzles.decoy, args)?;
        let solution = allocator.nil();
        
        Ok(CoinSpend::new(
            self.mix.decoy_coin.clone(),
            node_to_bytes(allocator, puzzle_reveal)?.into(),
            node_to_bytes(allocator, solution)?.into(),
        ))
    }
    
    fn create_decoy_value_spend(&self, allocator: &mut Allocator) -> Result<CoinSpend> {
        // Curry: (pk, msg)
        let decoy_value_pk = self.decoy_value_sk.public_key();
        
        let args = Self::build_curry_args_2(
            allocator,
            decoy_value_pk,
            self.decoy_value_msg.clone(),
        )?;
        
        let puzzle_reveal = self.curry_puzzle(allocator, &self.puzzles.decoy_value, args)?;
        let solution = allocator.nil();
        
        Ok(CoinSpend::new(
            self.mix.decoy_value_coin.clone(),
            node_to_bytes(allocator, puzzle_reveal)?.into(),
            node_to_bytes(allocator, solution)?.into(),
        ))
    }
}
