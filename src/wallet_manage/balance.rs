se crate::config::get_rpc_client;
use crate::utils::default_account;
use console::style;
use solana_sdk::signer::Signer;
use solana_sdk::{native_token::Sol, pubkey::Pubkey};
use std::str::FromStr;

#[derive(Debug, clap::Parser)]
pub struct BalanceArgs {
    #[clap(short, long)]
    address: Option<String>,
}

async fn check_balance(address: &str) -> anyhow::Result<()> {
    let client = get_rpc_client()?;
    let pubkey = Pubkey::from_str(&address)?;
    let balance = client.get_balance(&pubkey).await?;
    let lamports = Sol(balance);

    println!(
        "{} {}: {} {}",
        style("Balance for").cyan(),
        style(pubkey).yellow(),
        style(lamports).green().bold(),
        style("SOL").cyan()
    );
    Ok(())
}

pub async fn display_balance(args: &BalanceArgs) -> anyhow::Result<()> {
    if let Some(address) = args.address.as_ref() {
        check_balance(&address).await
    } else {
        check_default_balance().await
    }
}

async fn check_default_balance() -> anyhow::Result<()> {
    let keypair = default_account()?;

    check_balance(&keypair.pubkey().to_string()).await
}
