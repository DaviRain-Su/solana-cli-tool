use anyhow::Result;
use clap::Parser;
use solana_sdk::signature::{Keypair, Signer};

#[derive(Parser, Debug)]
pub struct NewWalletArgs {
    /// the keypair file name and save to ~/.config/solana/ if not specific name use public key as name
    #[clap(short, long)]
    keypair_file: Option<String>,
}

pub fn create_new_wallet(args: &NewWalletArgs) -> Result<()> {
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey();
    let keypair_file = args
        .keypair_file
        .clone()
        .unwrap_or_else(|| format!("{}.json", pubkey));

    // Get home directory and construct the full path
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let keypair_path = home_dir.join(".config").join("solana").join(&keypair_file);

    // Create directories if they don't exist
    std::fs::create_dir_all(keypair_path.parent().unwrap())?;

    println!("Your new wallet address is: {}", pubkey);
    println!("Saving keypair to {:?}", keypair_path);

    write_keypair_file(&keypair, &keypair_path.to_str().unwrap())?;
    Ok(())
}

pub fn write_keypair_file(keypair: &Keypair, filename: &str) -> Result<()> {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(filename)?;
    let secret_key_str = keypair.to_base58_string();
    // Create JSON structure
    let json_content = format!(r#"[{secret_key_str}]"#);
    file.write_all(&json_content.as_bytes())?;
    Ok(())
}
