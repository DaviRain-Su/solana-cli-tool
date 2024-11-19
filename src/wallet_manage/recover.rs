use crate::wallet_manage::new_wallet::write_keypair_file;
use bip39::{Language, Mnemonic};
use clap::Parser;
use console::{style, Emoji};
use solana_sdk::signature::SeedDerivable;
use solana_sdk::signature::Signer;
use solana_sdk::signer::keypair::Keypair;

#[derive(Parser)]
pub struct RestoreWalletArgs {
    #[clap(short, long)]
    mnemonic: String,
}

pub fn restore_wallet(args: &RestoreWalletArgs) -> anyhow::Result<()> {
    let mnemonic = Mnemonic::parse_in_normalized(Language::English, &args.mnemonic)?;
    let seed = mnemonic.to_seed("");

    // 直接从种子创建密钥对
    // TODO:(需要确定到底是什么方式倒入)
    let keypair = Keypair::from_seed(&seed).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    //let keypair = Keypair::from_bytes(&seed[..32]).map_err(|e| anyhow!(e.to_string()))?;
    let pubkey = keypair.pubkey();

    // 使用 emoji
    static WALLET: Emoji<'_, '_> = Emoji("💳 ", "");
    static SAVE: Emoji<'_, '_> = Emoji("💾 ", "");

    let keypair_file = format!("{}.json", pubkey);
    // Get home directory and construct the full path
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let keypair_path = home_dir.join(".config").join("solana").join(&keypair_file);

    println!("{} Created new wallet", WALLET);
    println!(
        "{} Saving keypair to {}",
        SAVE,
        style(keypair_path.display()).yellow()
    );

    // Create directories if they don't exist
    std::fs::create_dir_all(keypair_path.parent().unwrap())?;

    println!("Restored wallet address: {}", pubkey);
    write_keypair_file(&keypair, "restored_wallet.json")?;
    Ok(())
}

#[test]
fn test_restore_wallet() {
    let mnemonic =
        "struggle prepare gorilla eyebrow accuse scatter cabin civil much left vintage utility";
    let args = RestoreWalletArgs {
        mnemonic: mnemonic.to_string(),
    };
    restore_wallet(&args).unwrap();
}
