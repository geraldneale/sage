use anyhow::Result;
use clap::Parser;
use clvmr::{Allocator, run_program, ChiaDialect};
use sage_wallet::BlinkPuzzles;

#[derive(Debug, Parser)]
pub enum BlinkCommand {
    /// Load and verify all Blink Mojo puzzles
    LoadPuzzles,
    
    /// Test puzzle execution with sample data
    TestExecution {
        /// Which puzzle to test: needs_privacy, decoy_value, faucet, decoy
        #[clap(default_value = "needs_privacy")]
        puzzle: String,
    },
}

impl BlinkCommand {
    pub async fn handle(self) -> Result<()> {
        match self {
            Self::LoadPuzzles => {
                println!("🦄 Loading Blink Mojo puzzles...");
                let puzzles = BlinkPuzzles::load()?;
                println!("✓ needs_privacy: {} bytes", puzzles.needs_privacy.len());
                println!("✓ decoy_value: {} bytes", puzzles.decoy_value.len());
                println!("✓ faucet: {} bytes", puzzles.faucet.len());
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
                    "faucet" => &puzzles.faucet,
                    "decoy" => &puzzles.decoy,
                    _ => return Err(anyhow::anyhow!("Unknown puzzle: {}", puzzle)),
                };
                
                let mut allocator = Allocator::new();
                let puzzle_ptr = BlinkPuzzles::deserialize_puzzle(&mut allocator, puzzle_bytes)?;
                
                // Create test solution
                let test_pk = vec![0u8; 48];
                let test_msg = b"test message";
                let pk_ptr = allocator.new_atom(&test_pk)?;
                let msg_ptr = allocator.new_atom(test_msg)?;
                let nil = allocator.nil();
                let msg_list = allocator.new_pair(msg_ptr, nil)?;
                let solution_ptr = allocator.new_pair(pk_ptr, msg_list)?;
                
                // Execute with ChiaDialect
                let dialect = ChiaDialect::new(0);
                let result = run_program(&mut allocator, &dialect, puzzle_ptr, solution_ptr, 11_000_000_000)?;
                
                println!("✓ Execution successful!");
                println!("  Cost: {}", result.0);
                println!("  Result node: {:?}", result.1);
                println!("\n🎉 Puzzle execution verified!");
                Ok(())
            }
        }
    }
}
