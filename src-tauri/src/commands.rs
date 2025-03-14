use std::time::Duration;

use chia_wallet_sdk::decode_address;
use sage_api::{wallet_connect::*, *};
use sage_api_macro::impl_endpoints_tauri;
use sage_config::{NetworkConfig, WalletConfig};
use sage_rpc::start_rpc;
use specta::specta;
use tauri::{command, AppHandle, State};
use tokio::time::sleep;
use tracing::error;

use crate::{
    app_state::{self, AppState, Initialized, RpcTask},
    error::Result,
};

#[command]
#[specta]
pub async fn initialize(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    initialized: State<'_, Initialized>,
    rpc_task: State<'_, RpcTask>,
) -> Result<()> {
    let mut initialized = initialized.0.lock().await;

    if *initialized {
        return Ok(());
    }

    *initialized = true;

    let mut sage = state.lock().await;
    app_state::initialize(app_handle, &mut sage).await?;
    drop(sage);

    let app_state = (*state).clone();

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(3)).await;

            let app_state = app_state.lock().await;

            if let Err(error) = app_state.save_peers().await {
                error!("Error while saving peers: {error:?}");
            }

            drop(app_state);
        }
    });

    let app_state = state.lock().await;

    if app_state.config.rpc.run_on_startup {
        *rpc_task.0.lock().await = Some(tokio::spawn(start_rpc((*state).clone())));
    }

    Ok(())
}

impl_endpoints_tauri! {
    (repeat
        #[command]
        #[specta]
        pub async fn endpoint(state: State<'_, AppState>, req: Endpoint) -> Result<EndpointResponse> {
            Ok(state.lock().await.endpoint(req) maybe_await?)
        }
    )
}

#[command]
#[specta]
pub async fn validate_address(state: State<'_, AppState>, address: String) -> Result<bool> {
    let state = state.lock().await;
    let Some((_puzzle_hash, prefix)) = decode_address(&address).ok() else {
        return Ok(false);
    };
    Ok(prefix == state.network().address_prefix)
}

#[command]
#[specta]
pub async fn network_config(state: State<'_, AppState>) -> Result<NetworkConfig> {
    Ok(state.lock().await.config.network.clone())
}

#[command]
#[specta]
pub async fn wallet_config(state: State<'_, AppState>, fingerprint: u32) -> Result<WalletConfig> {
    Ok(state.lock().await.try_wallet_config(fingerprint).clone())
}

#[command]
#[specta]
pub async fn is_rpc_running(rpc_task: State<'_, RpcTask>) -> Result<bool> {
    Ok(rpc_task.0.lock().await.is_some())
}

#[command]
#[specta]
pub async fn start_rpc_server(
    state: State<'_, AppState>,
    rpc_task: State<'_, RpcTask>,
) -> Result<()> {
    let mut rpc_task = rpc_task.0.lock().await;
    *rpc_task = Some(tokio::spawn(start_rpc((*state).clone())));
    Ok(())
}

#[command]
#[specta]
pub async fn stop_rpc_server(rpc_task: State<'_, RpcTask>) -> Result<()> {
    let mut rpc_task = rpc_task.0.lock().await;
    if let Some(handle) = rpc_task.take() {
        handle.abort();
    }
    Ok(())
}

#[command]
#[specta]
pub async fn get_rpc_run_on_startup(state: State<'_, AppState>) -> Result<bool> {
    Ok(state.lock().await.config.rpc.run_on_startup)
}

#[command]
#[specta]
pub async fn set_rpc_run_on_startup(
    state: State<'_, AppState>,
    run_on_startup: bool,
) -> Result<()> {
    state.lock().await.config.rpc.run_on_startup = run_on_startup;
    state.lock().await.save_config()?;
    Ok(())
}
