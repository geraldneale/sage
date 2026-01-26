//! Tests for Blink Mojo curry implementation
//! 
//! All test vectors use FAKE addresses (0xAA...AA, 0xBB...BB) for testing curry logic only.

use crate::blink::*;
use chia::bls::PublicKey;
use chia::clvm_utils::tree_hash;
use clvmr::Allocator;
use hex_literal::hex;

// Test vectors with FAKE destinations
const TEST_PK: [u8; 48] = hex!("953c051e4a64299a41355a77f62e6ad353976ff68eee11da2790b7b204bee4e3343f9358c00f078c8924b013701325bf");
const TEST_MSG: [u8; 16] = hex!("746573745f6d6573736167655f313662");
const TEST_AMOUNT: u64 = 1000000000000;

// Fake destinations - NOT REAL ADDRESSES
const FAKE_ANON_DEST: [u8; 32] = hex!("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");  
const FAKE_KNOWN_DEST: [u8; 32] = hex!("BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB");

// Expected puzzle hashes
const FAUCET_HASH: [u8; 32] = hex!("b239af1c6c555b0863e366a2e5e5214250e718a08f8bab059ae4ea27a3f252d7");
const NEEDS_PRIVACY_HASH: [u8; 32] = hex!("a42f267a43c59af0fed2ad215ec37b9fded4ba6764ba4d0d2b51a3c2d0c10853");
const DECOY_HASH: [u8; 32] = hex!("320b2c41b44be952bf3fe65425cafcb262db6e6a2b4ca3b47d6e023f88bbdfea");
const DECOY_VALUE_HASH: [u8; 32] = hex!("a42f267a43c59af0fed2ad215ec37b9fded4ba6764ba4d0d2b51a3c2d0c10853");

#[test]
fn test_faucet_curry() {
    use chia::clvm_utils::CurriedProgram;
    use chia::clvm_traits::ToClvm;
    
    let pk = PublicKey::from_bytes(&TEST_PK).expect("valid public key");
    let msg = TEST_MSG.to_vec();
    let puzzles = BlinkPuzzles::load().expect("puzzles load");
    let mut allocator = Allocator::new();
    
    // Faucet goes to ANON destination
    let args = BlinkSettlement::build_curry_args_4(
        &mut allocator,
        pk,
        msg,
        TEST_AMOUNT,
        FAKE_ANON_DEST,
    ).expect("build args");
    
    let puzzle_ptr = BlinkPuzzles::deserialize_puzzle(&mut allocator, &puzzles.faucet).expect("deserialize");
    
    let curried = CurriedProgram {
        program: puzzle_ptr,
        args,
    };
    
    let curried_ptr = curried.to_clvm(&mut allocator).expect("curry");
    let puzzle_hash = tree_hash(&allocator, curried_ptr);
    
    assert_eq!(puzzle_hash.as_ref(), &FAUCET_HASH, "Faucet puzzle hash mismatch");
}

#[test]
fn test_needs_privacy_curry() {
    use chia::clvm_utils::CurriedProgram;
    use chia::clvm_traits::ToClvm;
    
    let pk = PublicKey::from_bytes(&TEST_PK).expect("valid public key");
    let msg = TEST_MSG.to_vec();
    let puzzles = BlinkPuzzles::load().expect("puzzles load");
    let mut allocator = Allocator::new();
    
    let args = BlinkSettlement::build_curry_args_2(&mut allocator, pk, msg).expect("build args");
    
    let puzzle_ptr = BlinkPuzzles::deserialize_puzzle(&mut allocator, &puzzles.needs_privacy).expect("deserialize");
    
    let curried = CurriedProgram {
        program: puzzle_ptr,
        args,
    };
    
    let curried_ptr = curried.to_clvm(&mut allocator).expect("curry");
    let puzzle_hash = tree_hash(&allocator, curried_ptr);
    
    assert_eq!(puzzle_hash.as_ref(), &NEEDS_PRIVACY_HASH, "Needs_privacy puzzle hash mismatch");
}

#[test]
fn test_decoy_curry() {
    use chia::clvm_utils::CurriedProgram;
    use chia::clvm_traits::ToClvm;
    
    let pk = PublicKey::from_bytes(&TEST_PK).expect("valid public key");
    let msg = TEST_MSG.to_vec();
    let puzzles = BlinkPuzzles::load().expect("puzzles load");
    let mut allocator = Allocator::new();
    
    // Decoy goes to KNOWN destination  
    let args = BlinkSettlement::build_curry_args_4(
        &mut allocator,
        pk,
        msg,
        TEST_AMOUNT,
        FAKE_KNOWN_DEST,
    ).expect("build args");
    
    let puzzle_ptr = BlinkPuzzles::deserialize_puzzle(&mut allocator, &puzzles.decoy).expect("deserialize");
    
    let curried = CurriedProgram {
        program: puzzle_ptr,
        args,
    };
    
    let curried_ptr = curried.to_clvm(&mut allocator).expect("curry");
    let puzzle_hash = tree_hash(&allocator, curried_ptr);
    
    assert_eq!(puzzle_hash.as_ref(), &DECOY_HASH, "Decoy puzzle hash mismatch");
}

#[test]
fn test_decoy_value_curry() {
    use chia::clvm_utils::CurriedProgram;
    use chia::clvm_traits::ToClvm;
    
    let pk = PublicKey::from_bytes(&TEST_PK).expect("valid public key");
    let msg = TEST_MSG.to_vec();
    let puzzles = BlinkPuzzles::load().expect("puzzles load");
    let mut allocator = Allocator::new();
    
    let args = BlinkSettlement::build_curry_args_2(&mut allocator, pk, msg).expect("build args");
    
    let puzzle_ptr = BlinkPuzzles::deserialize_puzzle(&mut allocator, &puzzles.decoy_value).expect("deserialize");
    
    let curried = CurriedProgram {
        program: puzzle_ptr,
        args,
    };
    
    let curried_ptr = curried.to_clvm(&mut allocator).expect("curry");
    let puzzle_hash = tree_hash(&allocator, curried_ptr);
    
    assert_eq!(puzzle_hash.as_ref(), &DECOY_VALUE_HASH, "Decoy_value puzzle hash mismatch");
}
