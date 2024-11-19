use crate::config::get_rpc_client;
use crate::utils::default_account;
use chrono::prelude::*;
use clap::Parser;
use console::style;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_response::RpcConfirmedTransactionStatusWithSignature;
use solana_sdk::{pubkey::Pubkey, signature::Signature, signer::Signer};
use std::{
    str::FromStr,
    time::{Duration, UNIX_EPOCH},
};

#[derive(Parser, Debug)]
pub struct AccountCreateTimestampArgs {
    /// Account address
    #[clap(short, long)]
    pub address: Option<String>,
}

pub async fn handle_account_create_timestamp(
    args: &AccountCreateTimestampArgs,
) -> anyhow::Result<()> {
    let client = get_rpc_client()?;
    if let Some(address) = args.address.as_ref() {
        let addr: Pubkey = address.parse()?;
        let datetime = get_account_creation_date(&client, &addr).await?;
        let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

        println!(
            "\n{} {}",
            style("Account:").bold().cyan(),
            style(addr.to_string()).yellow()
        );

        println!(
            "{} {} {}\n",
            style("Created on:").bold().cyan(),
            style(timestamp_str).green(),
            style("UTC").dim()
        );

        Ok(())
    } else {
        check_default_timestamp(&client).await
    }
}

async fn check_default_timestamp(client: &RpcClient) -> anyhow::Result<()> {
    let keypair = default_account()?;

    let datetime = get_account_creation_date(&client, &keypair.pubkey()).await?;
    let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

    println!(
        "\n{} {}",
        style("Account:").bold().cyan(),
        style(keypair.pubkey().to_string()).yellow()
    );

    println!(
        "{} {} {}\n",
        style("Created on:").bold().cyan(),
        style(timestamp_str).green(),
        style("UTC").dim()
    );

    Ok(())
}

pub async fn get_account_creation_date(
    rpc: &RpcClient,
    addr: &Pubkey,
) -> anyhow::Result<DateTime<Utc>> {
    let mut before = None;
    let mut earliest_sig: Option<RpcConfirmedTransactionStatusWithSignature> = None;

    loop {
        let sigs = fetch_signatures(rpc, addr, before).await?;

        if sigs.is_empty() {
            break;
        }

        let mut sigs_vec = sigs;
        sigs_vec.sort_by_key(|sig| sig.block_time);
        let current_earliest = sigs_vec.first().unwrap().clone();

        earliest_sig = Some(match earliest_sig {
            None => current_earliest.clone(),
            Some(prev) => {
                if prev.block_time > current_earliest.block_time {
                    current_earliest.clone()
                } else {
                    prev
                }
            }
        });

        if sigs_vec.len() < 1000 {
            break;
        }

        before = Some(Signature::from_str(&current_earliest.signature)?);
    }

    let status = earliest_sig.ok_or_else(|| anyhow::anyhow!("No signatures found!"))?;

    let d = UNIX_EPOCH
        + Duration::from_secs(
            status
                .block_time
                .ok_or_else(|| anyhow::anyhow!("Missing block time!"))?
                .try_into()?,
        );

    Ok(DateTime::<Utc>::from(d))
}

async fn fetch_signatures(
    rpc: &RpcClient,
    addr: &Pubkey,
    before: Option<Signature>,
) -> anyhow::Result<Vec<RpcConfirmedTransactionStatusWithSignature>> {
    Ok(rpc
        .get_signatures_for_address_with_config(
            addr,
            solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config {
                before,
                ..Default::default()
            },
        )
        .await?)
}
