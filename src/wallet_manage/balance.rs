use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signer::Signer;
use solana_sdk::{native_token::Sol, pubkey::Pubkey};
use std::str::FromStr;

async fn check_balance(address: &str) -> anyhow::Result<()> {
    let client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string()); // Use the appropriate network
    let pubkey = Pubkey::from_str(&address)?;
    let balance = client.get_balance(&pubkey).await?;
    let lamports = Sol(balance);

    println!("Balance for {}: {}", address, lamports);
    Ok(())
}

pub async fn display_balance(address: Option<&str>) -> anyhow::Result<()> {
    if let Some(address) = address {
        check_balance(&address).await
    } else {
        check_default_balance().await
    }
}

async fn check_default_balance() -> anyhow::Result<()> {
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

    check_balance(&keypair.pubkey().to_string()).await
}
