//! Tests for Blink Mojo curry implementation
//! 
//! All test vectors use FAKE addresses (0xAA...AA, 0xBB...BB) for testing curry logic only.

use crate::blink::*;
use chia::bls::PublicKey;
use chia::clvm_utils::tree_hash;
use chia_traits::Streamable;
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
const SOURCE_HASH: [u8; 32] = hex!("b239af1c6c555b0863e366a2e5e5214250e718a08f8bab059ae4ea27a3f252d7");
const NEEDS_PRIVACY_HASH: [u8; 32] = hex!("a42f267a43c59af0fed2ad215ec37b9fded4ba6764ba4d0d2b51a3c2d0c10853");
const DECOY_HASH: [u8; 32] = hex!("320b2c41b44be952bf3fe65425cafcb262db6e6a2b4ca3b47d6e023f88bbdfea");
const DECOY_VALUE_HASH: [u8; 32] = hex!("a42f267a43c59af0fed2ad215ec37b9fded4ba6764ba4d0d2b51a3c2d0c10853");

#[test]
fn test_source_curry() {
    use chia::clvm_utils::CurriedProgram;
    use chia::clvm_traits::ToClvm;
    
    let pk = PublicKey::from_bytes(&TEST_PK).expect("valid public key");
    let msg = TEST_MSG.to_vec();
    let _puzzles = BlinkPuzzles::load().expect("puzzles load");
    let mut allocator = Allocator::new();
    
    // Faucet goes to ANON destination
    let args = BlinkSettlement::build_curry_args_4(
        &mut allocator,
        pk,
        msg,
        TEST_AMOUNT,
        FAKE_ANON_DEST,
    ).expect("build args");
    
    let puzzle_ptr = BlinkPuzzles::deserialize_puzzle(&mut allocator, &puzzles.source).expect("deserialize");
    
    let curried = CurriedProgram {
        program: puzzle_ptr,
        args,
    };
    
    let curried_ptr = curried.to_clvm(&mut allocator).expect("curry");
    let puzzle_hash = tree_hash(&allocator, curried_ptr);
    
    assert_eq!(puzzle_hash.as_ref(), &SOURCE_HASH, "Faucet puzzle hash mismatch");
}

#[test]
fn test_needs_privacy_curry() {
    use chia::clvm_utils::CurriedProgram;
    use chia::clvm_traits::ToClvm;
    
    let pk = PublicKey::from_bytes(&TEST_PK).expect("valid public key");
    let msg = TEST_MSG.to_vec();
    let _puzzles = BlinkPuzzles::load().expect("puzzles load");
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
    let _puzzles = BlinkPuzzles::load().expect("puzzles load");
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
    let _puzzles = BlinkPuzzles::load().expect("puzzles load");
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

#[test]
fn test_build_spend_bundle() {
    use chia::protocol::Coin;
    use chia::bls::SecretKey;
    
    // Create 4 disposable keys
    let source_sk = SecretKey::from_bytes(&[1; 32]).unwrap();
    let needs_privacy_sk = SecretKey::from_bytes(&[2; 32]).unwrap();
    let decoy_sk = SecretKey::from_bytes(&[3; 32]).unwrap();
    let decoy_value_sk = SecretKey::from_bytes(&[4; 32]).unwrap();
    
    // Create mock coins with fake IDs
    let source_coin = Coin::new(
        [0xAA; 32].into(),  // parent
        [0xBB; 32].into(),  // puzzle_hash
        1000,               // amount
    );
    
    let needs_privacy_coin = Coin::new(
        [0xCC; 32].into(),
        [0xDD; 32].into(),
        500000000000,  // 0.5 XCH
    );
    
    let decoy_coin = Coin::new(
        [0xEE; 32].into(),
        [0xFF; 32].into(),
        100000000,  // Small amount
    );
    
    let decoy_value_coin = Coin::new(
        [0x11; 32].into(),
        [0x22; 32].into(),
        600000000000,  // >= needs_privacy (required!)
    );
    
    // Build the mix
    let mix = BlinkMix {
        source_coin,
        source_parent_id: [0xAA; 32],
        needs_privacy_coin,
        needs_privacy_value: 500000000000,
        needs_privacy_destination: FAKE_ANON_DEST,  // Using test constant
        decoy_coin,
        decoy_value_coin,
        decoy_value_amount: 600000000000,
        decoy_value_destination: FAKE_KNOWN_DEST,  // Using test constant
    };
    
    // Validate the mix
    mix.validate().expect("mix should be valid");
    
    // Load puzzles
    let _puzzles = BlinkPuzzles::load().expect("puzzles load");
    
    // Create settlement
    let settlement = BlinkSettlement::new(
        mix,
        source_sk,
        b"faucet_msg".to_vec(),
        needs_privacy_sk,
        b"needs_privacy_msg".to_vec(),
        decoy_sk,
        b"decoy_msg".to_vec(),
        decoy_value_sk,
        b"decoy_value_msg".to_vec(),
    ).expect("create settlement");
    
    // Create the spend bundle!
    let spend_bundle = settlement.build_spend_bundle().expect("create spend bundle");
    
    // Verify structure
    assert_eq!(spend_bundle.coin_spends.len(), 4, "Should have 4 coin spends");
    
    // Verify we have an aggregated signature
    let sig_bytes = spend_bundle.aggregated_signature.to_bytes();
    assert_ne!(sig_bytes, [0; 96], "Signature should not be all zeros");
    
    println!("✓ Spend bundle created successfully");
    println!("  - 4 coin spends");
    println!("  - Aggregated signature present");
    println!("  - Total size: {} bytes", {
        use chia::protocol::Bytes;
        let bundle_bytes: Bytes = spend_bundle.to_bytes().expect("serialize").into();
        bundle_bytes.len()
    });
}
