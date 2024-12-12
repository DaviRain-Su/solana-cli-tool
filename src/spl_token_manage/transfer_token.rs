use anyhow::Result;
use console::style;
use solana_sdk::transaction::Transaction;
use solana_sdk::{pubkey::Pubkey, signature::read_keypair_file, signer::Signer};
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;

use crate::config::get_rpc_client;
use crate::utils::default_account;

#[derive(Debug, clap::Parser)]
pub struct TransferTokenArgs {
    /// Source wallet keypair path (optional, uses default if not provided)
    #[clap(short, long)]
    pub source: Option<String>,

    /// Destination wallet address
    #[clap(short, long)]
    pub destination: String,

    /// Token mint address
    #[clap(short, long)]
    pub mint: String,

    /// Amount to transfer
    #[clap(short, long)]
    pub amount: f64,
}

pub async fn handle_transfer_token(args: &TransferTokenArgs) -> Result<()> {
    let client = get_rpc_client()?;

    // Get source keypair (default or specified)
    let source_keypair = match &args.source {
        Some(key) => read_keypair_file(key)
            .map_err(|e| anyhow::anyhow!("Failed to read keypair file: {}", e))?,
        None => default_account()?,
    };

    // Parse destination and mint addresses
    let destination = Pubkey::from_str(&args.destination)?;
    let mint = Pubkey::from_str(&args.mint)?;

    // Get token decimals for amount calculation
    let token_account = client.get_token_supply(&mint).await?;
    let decimals = token_account.decimals;
    let amount = (args.amount * 10f64.powi(decimals as i32)) as u64;

    // Get source and destination token accounts
    let source_ata = get_associated_token_address(&source_keypair.pubkey(), &mint);
    let destination_ata = get_associated_token_address(&destination, &mint);

    // Create transfer instruction
    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::id(),
        &source_ata,
        &destination_ata,
        &source_keypair.pubkey(),
        &[&source_keypair.pubkey()],
        amount,
    )?;

    // Create and send transaction
    let recent_blockhash = client.get_latest_blockhash().await?;
    let transaction = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&source_keypair.pubkey()),
        &[&source_keypair],
        recent_blockhash,
    );

    let signature = client.send_and_confirm_transaction(&transaction).await?;

    println!(
        "\n{} {} {} tokens from {} to {}",
        style("Successfully transferred").green(),
        style(args.amount).yellow(),
        style(&args.mint).cyan(),
        style(source_keypair.pubkey()).yellow(),
        style(&args.destination).yellow(),
    );
    println!("{}: {}", style("Transaction signature").cyan(), signature);

    Ok(())
}
