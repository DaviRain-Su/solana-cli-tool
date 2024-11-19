use solana_sdk::signature::Keypair;

/// default accout at ~/.config/solana/id.json
pub fn default_account() -> anyhow::Result<Keypair> {
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
    Ok(keypair)
}
