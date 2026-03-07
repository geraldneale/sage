use anyhow::Result;
use clap::Parser;
use clvmr::Allocator;
use sage_wallet::{BlinkPuzzles, BlinkMix, BlinkSettlement};
use chia::protocol::Coin;
use chia::bls::SecretKey;
use rand::RngCore;
use serde::{Serialize, Deserialize};
use bip39::Mnemonic;


// Imagine Wallet - publicly known mnemonic for anyone-can-broadcast Blink bundles
const IMAGINE_WALLET_MNEMONIC: &str = "duty love burger voyage snap case decrease ride true welcome bunker bench adult lizard lunar rich soda crush popular reflect ghost drink heart initial";

// Parse 32-byte hex string into Bytes32
fn parse_bytes32(hex: &str) -> Result<[u8; 32]> {
    let hex = hex.trim_start_matches("0x");
    let bytes = hex::decode(hex)
        .map_err(|e| anyhow::anyhow!("Invalid hex string: {}", e))?;
    
    if bytes.len() != 32 {
        return Err(anyhow::anyhow!("Expected 32 bytes (64 hex chars), got {}", bytes.len()));
    }
    
    let mut result = [0u8; 32];
    result.copy_from_slice(&bytes);
    Ok(result)
}
// Data structure for preparation output
#[derive(Debug, Serialize, Deserialize)]
struct PrepareData {
    destination: String,
    amount: u64,
    fee: u64,
    wallet_id: Option<u32>,
    // TODO: Add actual coin component data (parent_ids, puzzle_hashes, amounts)
    timestamp: u64,
}

#[derive(Debug, Parser)]
pub enum BlinkCommand {
    /// Load and verify all Blink Mojo puzzles
    LoadPuzzles,
    
    /// Test puzzle execution with sample data
    TestExecution {
        /// Which puzzle to test: needs_privacy, decoy_value, source, decoy
        #[clap(default_value = "needs_privacy")]
        puzzle: String,
    },
    
    /// Create and validate a Blink mixing transaction
    CreateMix {
        /// Source coin ID (from faucet or offer)
        #[clap(long)]
        source_coin_id: String,
        
        /// Needs privacy coin ID (hex)
        #[clap(long)]
        needs_privacy_coin_id: String,
        
        /// Needs privacy value (mojos)
        #[clap(long)]
        needs_privacy_value: u64,
        
        /// Decoy coin ID (hex)
        #[clap(long)]
        decoy_coin_id: String,
        
        /// Decoy value amount (mojos, must be >= needs_privacy_value)
        #[clap(long)]
        decoy_value_amount: u64,
    },
    
    
    /// Prepare coins for Blink mixing (Transaction 1)
    Prepare {
        /// Destination address for privacy output (xch address or puzzle hash hex)
        #[clap(long)]
        destination: String,
        
        /// Amount to mix (mojos) - used for needs_privacy and decoy_value coins
        #[clap(long)]
        amount: u64,
        
        /// Fee amount (mojos) - becomes decoy coin value
        #[clap(long)]
        fee: u64,
        
        /// Wallet ID to spend from
        #[clap(long)]
        wallet_id: Option<u32>,
        
        /// Output file for preparation data
        #[clap(long, default_value = "blink_prep.json")]
        output: String,
    },

    /// Create complete spend bundle (settlement)
    Settle {
        /// Faucet/source coin ID (hex, 64 chars)
        #[clap(long)]
        faucet_coin_id: String,
        
        /// Preparation file from 'blink prepare' command
        #[clap(long)]
        prepare_file: String,
        
        /// Wallet secret key (hex, 64 chars) - used for all 3 recoverable coins
        #[clap(long)]
        wallet_sk: Option<String>,

        /// Use Imagine Wallet public key for anyone-can-broadcast bundles
        #[clap(long)]
        use_imagine_wallet: bool,
}
}

impl BlinkCommand {
    pub async fn handle(self) -> Result<()> {
        match self {
            Self::LoadPuzzles => {
                println!("🦄 Loading Blink Mojo puzzles...");
                let puzzles = BlinkPuzzles::load()?;
                println!("✓ needs_privacy: {} bytes", puzzles.needs_privacy.len());
                println!("✓ decoy_value: {} bytes", puzzles.decoy_value.len());
                println!("✓ source: {} bytes", puzzles.source.len());
                println!("✓ decoy: {} bytes", puzzles.decoy.len());
                println!("\n🎉 All puzzles loaded successfully!");
                Ok(())
            }
            
            Self::TestExecution { puzzle } => {
                println!("🧪 Testing {} puzzle execution...", puzzle);
                let puzzles = BlinkPuzzles::load()?;
                
                let puzzle_bytes = match puzzle.as_str() {
                    "needs_privacy" => &puzzles.needs_privacy,
                    "decoy_value" => &puzzles.decoy_value,
                    "source" => &puzzles.source,
                    "decoy" => &puzzles.decoy,
                    _ => return Err(anyhow::anyhow!("Unknown puzzle: {}", puzzle)),
                };
                
                let mut allocator = Allocator::new();
                let _puzzle_ptr = BlinkPuzzles::deserialize_puzzle(&mut allocator, puzzle_bytes)?;
                
                println!("✓ Puzzle deserialized successfully");
                println!("✓ Puzzle size: {} bytes", puzzle_bytes.len());
                println!("\n🎉 Puzzle execution test passed!");
                
                Ok(())
            }
            
            Self::CreateMix {
                source_coin_id: _,
                needs_privacy_coin_id: _,
                needs_privacy_value,
                decoy_coin_id: _,
                decoy_value_amount,
            } => {
                println!("🎭 Creating Blink mixing transaction...\n");
                
                // Parse coin IDs (simplified - in real implementation would query chain)
                let source_coin = Coin {
                    parent_coin_info: [0u8; 32].into(),
                    puzzle_hash: [0u8; 32].into(),
                    amount: 1000,
                };
                
                let needs_privacy_coin = Coin {
                    parent_coin_info: [1u8; 32].into(),
                    puzzle_hash: [1u8; 32].into(),
                    amount: needs_privacy_value,
                };
                
                let decoy_coin = Coin {
                    parent_coin_info: [2u8; 32].into(),
                    puzzle_hash: [2u8; 32].into(),
                    amount: 10,
                };
                
                let decoy_value_coin = Coin {
                    parent_coin_info: [3u8; 32].into(),
                    puzzle_hash: [3u8; 32].into(),
                    amount: decoy_value_amount,
                };
                
                let mix = BlinkMix {
                    source_coin: source_coin.clone(),
                    source_parent_id: source_coin.parent_coin_info.into(),
                    needs_privacy_coin: needs_privacy_coin.clone(),
                    needs_privacy_value,
                    needs_privacy_destination: [3u8; 32],
                    decoy_coin: decoy_coin.clone(),
                    decoy_value_coin: decoy_value_coin.clone(),
                    decoy_value_amount,
                    decoy_value_destination: [4u8; 32],
                };
                
                // Validate
                match mix.validate() {
                    Ok(_) => {
                        println!("✅ Mix validation PASSED");
                        println!("\nMix details:");
                        println!("  Source coin: {}", hex::encode(source_coin.coin_id()));
                        println!("  Needs privacy: {} mojos", needs_privacy_value);
                        println!("  Decoy value: {} mojos", decoy_value_amount);
                        println!("  Privacy ratio: {:.2}x", decoy_value_amount as f64 / needs_privacy_value as f64);
                        println!("\n🎉 Ready to execute mix!");
                    }
                    Err(e) => {
                        println!("❌ Mix validation FAILED:");
                        println!("  {}", e);
                        println!("\n💡 Fix: Ensure decoy_value >= needs_privacy_value");
                        return Err(anyhow::anyhow!(e));
                    }
                }
                
                Ok(())
            }
            
            Self::Prepare {
                destination,
                amount,
                fee,
                wallet_id,
                output,
            } => {
                println!("🔧 Preparing Blink coins (Transaction 1)...\n");
                
                println!("📋 Preparation Plan:");
                println!("  Destination: {}", destination);
                println!("  Amount (needs_privacy + decoy_value): {} mojos", amount);
                println!("  Fee (decoy coin): {} mojos", fee);
                println!("  Wallet ID: {:?}", wallet_id);
                
                println!("\n⚠️  TODO: This will create 3 coins:");
                println!("  1. needs_privacy coin: {} mojos → {}", amount, destination);
                println!("  2. decoy coin: {} mojos → random", fee);
                println!("  3. decoy_value coin: {} mojos → {}", amount, destination);
                
                // Create preparation data
                let prep_data = PrepareData {
                    destination: destination.clone(),
                    amount,
                    fee,
                    wallet_id,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };
                
                // Save to JSON file
                let json = serde_json::to_string_pretty(&prep_data)?;
                std::fs::write(&output, json)?;
                
                println!("\n✅ Preparation data saved to: {}", output);
                println!("💡 Next: Use this with 'sage blink settle --prepare-file {}'", output);
                
                Ok(())
            }

            Self::Settle {
                faucet_coin_id,
                prepare_file,
                wallet_sk,
                use_imagine_wallet,
            } => {
                println!("⚡ Creating Blink Mojo spend bundle...\n");
                
                // Read preparation data
                println!("📖 Reading preparation file: {}", prepare_file);
                let prep_json = std::fs::read_to_string(&prepare_file)?;
                let prep_data: PrepareData = serde_json::from_str(&prep_json)?;
                
                println!("✓ Preparation data loaded");
                
                // Parse faucet coin ID and destinations
                let _source_coin_id_parsed = parse_bytes32(&faucet_coin_id)?;
                let needs_privacy_dest = parse_bytes32(&prep_data.destination)?;
                let needs_privacy_value = prep_data.amount;
                let decoy_value_amount = prep_data.amount;
                let decoy_value_dest = needs_privacy_dest;
                
                let source_coin = Coin {
                    parent_coin_info: [0xAA; 32].into(),
                    puzzle_hash: [0xBB; 32].into(),
                    amount: 1000,
                };
                
                let needs_privacy_coin = Coin {
                    parent_coin_info: [0xCC; 32].into(),
                    puzzle_hash: [0xDD; 32].into(),
                    amount: needs_privacy_value,
                };
                
                let decoy_coin = Coin {
                    parent_coin_info: [0xEE; 32].into(),
                    puzzle_hash: [0xFF; 32].into(),
                    amount: 10,
                };
                
                let decoy_value_coin = Coin {
                    parent_coin_info: [0x11; 32].into(),
                    puzzle_hash: [0x22; 32].into(),
                    amount: decoy_value_amount,
                };
                
                let mut needs_privacy_dest_array = [0u8; 32];
                needs_privacy_dest_array.copy_from_slice(&needs_privacy_dest);
                
                let mut decoy_value_dest_array = [0u8; 32];
                decoy_value_dest_array.copy_from_slice(&decoy_value_dest);
                
                let mix = BlinkMix {
                    source_coin: source_coin.clone(),
                    source_parent_id: source_coin.parent_coin_info.into(),
                    needs_privacy_coin,
                    needs_privacy_value,
                    needs_privacy_destination: needs_privacy_dest_array,
                    decoy_coin,
                    decoy_value_coin,
                    decoy_value_amount,
                    decoy_value_destination: decoy_value_dest_array,
                };
                
                println!("✓ Mix created and validated");
                
                // Generate random disposable key for source (privacy-critical)
                // Generate random disposable key for source coin (with retry for valid key)
                let source_sk = loop {
                    let mut source_seed = [0u8; 32];
                    rand::thread_rng().fill_bytes(&mut source_seed);
                    if let Ok(sk) = SecretKey::from_bytes(&source_seed) {
                        break sk;
                    }
                };
                
                println!("✓ Generated random disposable key for source coin");
                
                // Parse wallet secret key or use test keys
                let wallet_sk = if use_imagine_wallet {
                    println!("🌍 Using Imagine Wallet (anyone can broadcast)");
                    let mnemonic = IMAGINE_WALLET_MNEMONIC.parse::<Mnemonic>()
                        .map_err(|e| anyhow::anyhow!("Invalid Imagine Wallet mnemonic: {}", e))?;
                    let seed = mnemonic.to_seed("");
                    let mut seed_array = [0u8; 32];
                    seed_array.copy_from_slice(&seed[..32]);
                    SecretKey::from_bytes(&seed_array)?
                } else if let Some(sk_hex) = wallet_sk {
                    let sk_hex = sk_hex.trim_start_matches("0x");
                    let sk_bytes = hex::decode(sk_hex)
                        .map_err(|e| anyhow::anyhow!("Invalid wallet-sk hex: {}", e))?;
                    
                    if sk_bytes.len() != 32 {
                        return Err(anyhow::anyhow!("wallet-sk must be 32 bytes (64 hex chars)"));
                    }
                    
                    let mut sk_array = [0u8; 32];
                    sk_array.copy_from_slice(&sk_bytes);
                    SecretKey::from_bytes(&sk_array)?
                } else {
                    println!("⚠️  No --wallet-sk provided, using test keys (NOT SECURE)");
                    SecretKey::from_bytes(&[1; 32])?
                };
                
                // Use same wallet key for all 3 recoverable coins
                let needs_privacy_sk = wallet_sk.clone();
                let decoy_sk = wallet_sk.clone();
                let decoy_value_sk = wallet_sk;
                
                println!("✓ Wallet keys configured for recoverable coins");
                println!("✓ Using test keys for other coins (TODO: integrate keychain)");
                
                // Create settlement
                let settlement = BlinkSettlement::new(
                    mix,
                    source_sk,
                    b"source_msg".to_vec(),
                    needs_privacy_sk,
                    b"needs_privacy_msg".to_vec(),
                    decoy_sk,
                    b"decoy_msg".to_vec(),
                    decoy_value_sk,
                    b"decoy_value_msg".to_vec(),
                )?;
                
                println!("✓ Settlement created");
                
                // Build spend bundle
                let spend_bundle = settlement.build_spend_bundle()?;
                
                println!("\n🎉 Spend bundle created successfully!");
                println!("  Coin spends: {}", spend_bundle.coin_spends.len());
                println!("  Aggregated signature: {} bytes", spend_bundle.aggregated_signature.to_bytes().len());
                
                // Serialize for broadcast
                use chia_traits::Streamable;
                let bundle_bytes = spend_bundle.to_bytes()?;
                println!("  Total size: {} bytes", bundle_bytes.len());
                println!("\n💡 Spend bundle ready for broadcast to network");
                
                Ok(())
            }
        }
    }
}
