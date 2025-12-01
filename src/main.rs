use dotenv::dotenv;
use raydium_pump_snipe_bot::{
    common::{
        logger::Logger,
        utils::{
            create_nonblocking_rpc_client, create_rpc_client, import_env_var, import_wallet,
            AppState,
        },
    },
    engine::monitor::{pumpfun_monitor, raydium_monitor},
    services::jito,
};
use solana_sdk::signer::Signer;

#[tokio::main]
async fn main() {
    let logger = Logger::new("[INIT] => ".to_string());

    dotenv().ok();
    let rpc_wss = import_env_var("RPC_WSS");
    let rpc_client = create_rpc_client().unwrap();
    let rpc_nonblocking_client = create_nonblocking_rpc_client().await.unwrap();
    let wallet = import_wallet().unwrap();
    let wallet_cloned = wallet.clone();

    let state = AppState {
        rpc_client,
        rpc_nonblocking_client,
        wallet,
    };
    let slippage = import_env_var("SLIPPAGE").parse::<u64>().unwrap_or(5);
    let use_jito = true;
    if use_jito {
        jito::init_tip_accounts().await.unwrap();
    }

    logger.log(format!(
        "Successfully Set the environment variables.\n\t\t\t\t [Web Socket RPC]: {},\n\t\t\t\t [Wallet]: {:?},\n\t\t\t\t [Slippage]: {}\n", 
        rpc_wss, wallet_cloned.pubkey(), slippage
    ));
    // raydium_monitor(&rpc_wss, state, slippage, use_jito).await;
    pumpfun_monitor(&rpc_wss, state, slippage, use_jito).await;
}
