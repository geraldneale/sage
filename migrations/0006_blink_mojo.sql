/*
 * Blink Mojo privacy protocol tables
 * 
 * These tables track Blink preparation and settlement transactions
 * separately from Sage's core coin tracking.
 */

-- Track Blink preparation transactions (creates 3 coins)
CREATE TABLE blink_preparations (
  id INTEGER NOT NULL PRIMARY KEY,
  destination_puzzle_hash BLOB NOT NULL,
  needs_privacy_amount INTEGER NOT NULL,
  decoy_fee INTEGER NOT NULL,
  preparation_tx_hash BLOB UNIQUE,
  created_timestamp INTEGER NOT NULL,
  is_broadcast INTEGER NOT NULL DEFAULT 0
);

-- Track individual Blink coins created in preparation phase
CREATE TABLE blink_coins (
  id INTEGER NOT NULL PRIMARY KEY,
  preparation_id INTEGER NOT NULL,
  coin_id BLOB NOT NULL UNIQUE,
  coin_type TEXT NOT NULL, -- 'needs_privacy', 'decoy', 'decoy_value'
  amount INTEGER NOT NULL,
  puzzle_hash BLOB NOT NULL,
  is_spent INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (preparation_id) REFERENCES blink_preparations(id)
);

-- Track Blink settlement transactions (spends 4 coins atomically)
CREATE TABLE blink_settlements (
  id INTEGER NOT NULL PRIMARY KEY,
  preparation_id INTEGER NOT NULL,
  faucet_coin_id BLOB NOT NULL,
  settlement_tx_hash BLOB UNIQUE,
  created_timestamp INTEGER NOT NULL,
  is_broadcast INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (preparation_id) REFERENCES blink_preparations(id)
);

-- Indexes for efficient lookups
CREATE INDEX idx_blink_coins_preparation ON blink_coins(preparation_id);
CREATE INDEX idx_blink_coins_type ON blink_coins(coin_type);
CREATE INDEX idx_blink_settlements_preparation ON blink_settlements(preparation_id);
