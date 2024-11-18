use super::new_wallet::write_keypair_file;
use solana_sdk::signature::{Keypair, Signer};

// 在 wallet_manage.rs 中添加处理函数
pub fn recover_from_private_key(private_key: &str) -> anyhow::Result<()> {
    // 创建 Keypair
    let keypair = Keypair::from_base58_string(&private_key);
    let pubkey = keypair.pubkey();

    // 构造保存路径
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let keypair_path = home_dir
        .join(".config")
        .join("solana")
        .join(format!("{}.json", pubkey));

    // 创建目录
    std::fs::create_dir_all(keypair_path.parent().unwrap())?;

    println!("Recovered wallet address: {}", pubkey);
    println!("Saving keypair to {:?}", keypair_path);

    // 保存私钥
    write_keypair_file(&keypair, keypair_path.to_str().unwrap())?;
    Ok(())
}
