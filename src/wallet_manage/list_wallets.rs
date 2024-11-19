use console::style;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use std::fs;

pub fn list_all_wallets() -> anyhow::Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Unable to get home directory"))?;
    let config_path = home.join(".config/solana");

    println!("\n{}", style("🔑 Solana Wallets").bold().underlined());

    let paths = fs::read_dir(config_path)?;
    let mut found = false;

    for path in paths {
        let path = path?.path();
        if let Some(extension) = path.extension() {
            if extension == "json" {
                if let Some(file_name) = path.file_name() {
                    // 读取并解析密钥文件
                    if let Ok(content) = fs::read_to_string(&path) {
                        // 移除可能的百分号和空白字符
                        let content = content.trim().trim_end_matches('%');

                        // 尝试将字符串解析为字节数组
                        if let Ok(bytes) = content
                            .trim_start_matches('[')
                            .trim_end_matches(']')
                            .split(',')
                            .map(|s| s.trim().parse::<u8>())
                            .collect::<Result<Vec<u8>, _>>()
                        {
                            found = true;

                            // 从字节数组创建 Keypair
                            if let Ok(keypair) = Keypair::from_bytes(&bytes) {
                                println!(
                                    "{} {}",
                                    style(format!("→ {}", file_name.to_string_lossy())).cyan(),
                                    style(keypair.pubkey().to_string()).yellow()
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    if !found {
        println!("{}", style("No wallet config files found").dim().italic());
    }

    println!(); // 添加空行
    Ok(())
}
