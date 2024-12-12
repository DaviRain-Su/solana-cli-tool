use anyhow::Result;
use clap::Parser;
use console::style;
use solana_sdk::{pubkey::Pubkey, signature::read_keypair_file, signer::Signer};
use spl_associated_token_account::get_associated_token_address;
use solana_sdk::transaction::Transaction;
use std::str::FromStr;

use crate::config::get_rpc_client;
use crate::utils::default_account;

#[derive(Parser, Debug)]
pub struct TransferTokenArgs {
    /// Destination wallet address
    #[clap(short, long)]
    pub destination: String,

    /// Token mint address
    #[clap(short, long)]
    pub mint: String,

    /// Amount to transfer
    #[clap(short, long)]
    pub amount: f64,

    /// Source wallet (optional, defaults to ~/.config/solana/id.json)
    #[clap(short, long)]
    pub source: Option<String>,
}

pub async fn handle_transfer_token(args: &TransferTokenArgs) -> Result<()> {
    let client = get_rpc_client()?;
    let mint = Pubkey::from_str(&args.mint)?;
    let destination = Pubkey::from_str(&args.destination)?;

    // Get source keypair (default or specified)
    let source_keypair = match &args.source {
        Some(key) => read_keypair_file(key).map_err(|e| anyhow::anyhow!("Failed to read keypair file: {}", e))?,
        None => default_account()?,
    };

    // Get source ATA
    let source_ata = get_associated_token_address(&source_keypair.pubkey(), &mint);

    // Get destination ATA
    let dest_ata = get_associated_token_address(&destination, &mint);

    // Get token decimals for amount calculation
    let token_account = client.get_token_account(&source_ata).await?;
    let decimals = token_account
        .ok_or_else(|| anyhow::anyhow!("Source token account not found"))?
        .token_amount
        .decimals;

    // Calculate amount with decimals
    let amount = (args.amount * 10f64.powi(decimals as i32)) as u64;

    // Build transfer instruction
    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::id(),
        &source_ata,
        &dest_ata,
        &source_keypair.pubkey(),
        &[&source_keypair.pubkey()],
        amount,
    )?;

    // Send and confirm transaction
    let recent_blockhash = client.get_latest_blockhash().await?;
    let transaction = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&source_keypair.pubkey()),
        &[&source_keypair],
        recent_blockhash,
    );

    let signature = client.send_and_confirm_transaction(&transaction).await?;

    // Print success message
    println!(
        "{} {} tokens transferred to {}",
        style("✓").green(),
        style(args.amount.to_string()).yellow(),
        style(destination.to_string()).yellow()
    );
    println!(
        "{} Transaction signature: {}",
        style("📝").bold(),
        style(signature.to_string()).yellow()
    );

    Ok(())
}
