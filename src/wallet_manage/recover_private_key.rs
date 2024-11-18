use super::new_wallet::write_keypair_file;
use console::{style, Emoji};
use solana_sdk::signature::{Keypair, Signer};

// 在 wallet_manage.rs 中添加处理函数
pub fn recover_from_private_key(private_key: &str) -> anyhow::Result<()> {
    // 创建 Keypair
    let keypair = Keypair::from_base58_string(&private_key);
    let pubkey = keypair.pubkey();

    // 使用不同颜色和样式
    println!(
        "{} {}",
        style("Your wallet address is:").cyan().bold(),
        style(pubkey).green()
    );

    // 使用 emoji
    static WALLET: Emoji<'_, '_> = Emoji("💳 ", "");
    static SAVE: Emoji<'_, '_> = Emoji("💾 ", "");

    // 构造保存路径
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let keypair_path = home_dir
        .join(".config")
        .join("solana")
        .join(format!("{}.json", pubkey));

    println!("{} Recover wallet", WALLET);
    println!(
        "{} Saving keypair to {}",
        SAVE,
        style(keypair_path.display()).yellow()
    );

    // 创建目录
    std::fs::create_dir_all(keypair_path.parent().unwrap())?;

    // 保存私钥
    write_keypair_file(&keypair, keypair_path.to_str().unwrap())?;

    // 可以添加成功标记
    println!(
        "{} {}",
        style("✔").green(),
        style("Wallet recover successfully!").green().bold()
    );
    Ok(())
}
