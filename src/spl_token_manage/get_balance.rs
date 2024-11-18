use crate::config::get_rpc_client;
use clap::Parser;
use console::style;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signer::Signer};

/// get spl token balance
#[derive(Parser, Debug)]
pub struct GetBalanceArgs {
    #[clap(short, long)]
    wallet_pubkey: Option<String>,
    #[clap(short, long)]
    mint_account_pubkey: String,
}

pub async fn handle_get_balance(args: &GetBalanceArgs) -> anyhow::Result<()> {
    let client = get_rpc_client()?;
    let mint_id: Pubkey = args.mint_account_pubkey.parse()?;

    if let Some(wallet_pubkey) = &args.wallet_pubkey {
        let target: Pubkey = wallet_pubkey.parse()?;
        let addr = spl_associated_token_account::get_associated_token_address(&target, &mint_id);
        let balance = client.get_token_account_balance(&addr).await?;

        println!(
            "{} {}",
            style("Wallet pubkey:").bold().cyan(),
            style(wallet_pubkey.to_string()).yellow()
        );

        println!(
            "{} {}",
            style("Mint account:").bold().cyan(),
            style(mint_id.to_string()).yellow()
        );

        println!(
            "{} {}",
            style("Associated token account:").bold().cyan(),
            style(addr.to_string()).yellow()
        );

        println!(
            "{} {}",
            style("Amount:").bold().cyan(),
            style(balance.ui_amount_string).green()
        );

        println!(
            "{} {}",
            style("Decimals:").bold().cyan(),
            style(balance.decimals.to_string()).green()
        );
        Ok(())
    } else {
        // use default wallet
        check_default_balance(&client, &mint_id).await
    }
}

async fn check_default_balance(client: &RpcClient, mint_id: &Pubkey) -> anyhow::Result<()> {
    // default address at ~/.config/solana/id.json
    // 构造保存路径
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let keypair_path = home_dir
        .join(".config")
        .join("solana")
        .join(format!("id.json"));

    // read keypair from file
    let keypair = solana_sdk::signature::read_keypair_file(&keypair_path.to_str().unwrap())
        .map_err(|e| anyhow::anyhow!("{}", e.to_string()))?;

    let target: Pubkey = keypair.pubkey();
    let addr = spl_associated_token_account::get_associated_token_address(&target, &mint_id);
    let balance = client.get_token_account_balance(&addr).await?;

    println!(
        "{} {}",
        style("Wallet pubkey:").bold().cyan(),
        style(target.to_string()).yellow()
    );

    println!(
        "{} {}",
        style("Mint account:").bold().cyan(),
        style(mint_id.to_string()).yellow()
    );

    println!(
        "{} {}",
        style("Associated token account:").bold().cyan(),
        style(addr.to_string()).yellow()
    );

    println!(
        "{} {}",
        style("Amount:").bold().cyan(),
        style(balance.ui_amount_string).green()
    );

    println!(
        "{} {}",
        style("Decimals:").bold().cyan(),
        style(balance.decimals.to_string()).green()
    );

    Ok(())
}
