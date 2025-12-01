use std::{env, sync::Arc, time::Duration};

use anyhow::Result;
use jito_json_rpc_client::jsonrpc_client::rpc_client::RpcClient as JitoRpcClient;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction,
    signature::Keypair,
    signer::Signer,
    system_transaction,
    transaction::{Transaction, VersionedTransaction},
};
use spl_token::ui_amount_to_amount;

use std::str::FromStr;
use tokio::time::Instant;

use crate::{
    common::logger::Logger,
    services::jito::{self, get_tip_account, get_tip_value, wait_for_bundle_confirmation},
};

// prioritization fee = UNIT_PRICE * UNIT_LIMIT
fn get_unit_price() -> u64 {
    env::var("UNIT_PRICE")
        .ok()
        .and_then(|v| u64::from_str(&v).ok())
        .unwrap_or(1)
}

fn get_unit_limit() -> u32 {
    env::var("UNIT_LIMIT")
        .ok()
        .and_then(|v| u32::from_str(&v).ok())
        .unwrap_or(300_000)
}

pub async fn new_signed_and_send(
    client: &RpcClient,
    keypair: &Keypair,
    mut instructions: Vec<Instruction>,
    use_jito: bool,
    logger: &Logger,
) -> Result<Vec<String>> {
    let unit_price = get_unit_price();
    let unit_limit = get_unit_limit();
    // If not using Jito, manually set the compute unit price and limit
    if !use_jito {
        let modify_compute_units =
            solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_price(
                unit_price,
            );
        let add_priority_fee =
            solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(
                unit_limit,
            );
        instructions.insert(0, modify_compute_units);
        instructions.insert(1, add_priority_fee);
    }
    // send init tx
    let recent_blockhash = client.get_latest_blockhash()?;
    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&keypair.pubkey()),
        &vec![keypair],
        recent_blockhash,
    );

    let start_time = Instant::now();
    let mut txs = vec![];
    if use_jito {
        // jito
        let tip_account = get_tip_account().await?;
        let jito_client = Arc::new(JitoRpcClient::new(format!(
            "{}/api/v1/bundles",
            *jito::BLOCK_ENGINE_URL
        )));
        // jito tip, the upper limit is 0.1
        let mut tip = get_tip_value().await?;
        tip = tip.min(0.1);
        let tip_lamports = ui_amount_to_amount(tip, spl_token::native_mint::DECIMALS);
        logger.log(format!(
            "tip account: {}, tip(sol): {}, lamports: {}",
            tip_account, tip, tip_lamports
        ));
        // tip tx
        let bundle: Vec<VersionedTransaction> = vec![
            VersionedTransaction::from(txn),
            VersionedTransaction::from(system_transaction::transfer(
                keypair,
                &tip_account,
                tip_lamports,
                recent_blockhash,
            )),
        ];
        let bundle_id = jito_client.send_bundle(&bundle).await?;
        logger.log(format!("bundle_id: {}", bundle_id));

        txs = wait_for_bundle_confirmation(
            move |id: String| {
                let client = Arc::clone(&jito_client);
                async move {
                    let response = client.get_bundle_statuses(&[id]).await;
                    let statuses = response.inspect_err(|err| {
                        logger.log(format!("Error fetching bundle status: {:?}", err));
                    })?;
                    Ok(statuses.value)
                }
            },
            bundle_id,
            Duration::from_millis(1000),
            Duration::from_secs(10),
        )
        .await?;
    } else {
        let sig = common::rpc::send_txn(client, &txn, true)?;
        logger.log(format!("signature: {:#?}", sig));
        txs.push(sig.to_string());
    }

    logger.log(format!("tx ellapsed: {:?}", start_time.elapsed()));
    Ok(txs)
}
