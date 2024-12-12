use clap::Parser;
use console::style;
use solana_sdk::signature::Signer;
use solana_sdk::signer::keypair::read_keypair_file;

/// display wallet from base58 encoded private key
#[derive(Debug, Parser)]
pub struct DisplayPrivateKeyArgs {
    #[clap(short, long)]
    wallet_name: Option<String>, // 改为 Option
}

pub fn display_private_key(args: &DisplayPrivateKeyArgs) -> anyhow::Result<()> {
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let default_keypair_path = home_dir.join(".config").join("solana").join("id.json");

    let keypair_path = match &args.wallet_name {
        Some(from) => {
            // 如果提供了 from，构造对应的路径
            home_dir
                .join(".config")
                .join("solana")
                .join(format!("{}.json", from))
        }
        None => default_keypair_path,
    };

    // 读取 keypair
    let keypair = read_keypair_file(keypair_path.to_str().unwrap())
        .map_err(|_e| anyhow::anyhow!("Failed to read keypair from {:?}", keypair_path))?;

    // 使用不同颜色和样式
    println!(
        "{} {}",
        style("Your wallet address is:").cyan().bold(),
        style(keypair.pubkey().to_string()).green()
    );
    println!(
        "{} {}",
        style("Your wallet private key is:").cyan().bold(),
        style(keypair.to_base58_string()).green()
    );

    Ok(())
}
