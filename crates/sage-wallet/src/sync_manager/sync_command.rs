use std::{net::IpAddr, sync::Arc};

use chia::protocol::{Bytes32, Message};
use chia_wallet_sdk::client::Network;

use crate::Wallet;

#[derive(Debug)]
pub enum SyncCommand {
    SwitchWallet {
        wallet: Option<Arc<Wallet>>,
    },
    SwitchNetwork {
        network_id: String,
        network: Network,
    },
    HandleMessage {
        ip: IpAddr,
        message: Message,
    },
    ConnectPeer {
        ip: IpAddr,
        user_managed: bool,
    },
    SubscribeCoins {
        coin_ids: Vec<Bytes32>,
    },
    SubscribePuzzles {
        puzzle_hashes: Vec<Bytes32>,
    },
    ConnectionClosed(IpAddr),
    SetTargetPeers(usize),
    SetDiscoverPeers(bool),
}
