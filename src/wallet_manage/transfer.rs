use crate::config::get_rpc_client;
use anyhow::Context;
use clap::Parser;
use console::{style, Term};
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::native_token::sol_to_lamports;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use solana_sdk::signer::keypair::read_keypair_file;
use solana_sdk::system_instruction;
use solana_sdk::transaction::Transaction;
use std::str::FromStr;

#[derive(Parser, Debug)]
pub struct TransferArgs {
    #[clap(short, long)]
    from: Option<String>, // 改为 Option
    #[clap(short, long)]
    to: String,
    #[clap(short, long)]
    amount: f64,
}

pub async fn transfer_sol(args: &TransferArgs) -> anyhow::Result<()> {
    let client = get_rpc_client()?;

    // 获取 keypair 文件路径
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let default_keypair_path = home_dir.join(".config").join("solana").join("id.json");

    let keypair_path = match &args.from {
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
    let from_keypair = read_keypair_file(keypair_path.to_str().unwrap())
        .map_err(|_e| anyhow::anyhow!("Failed to read keypair from {:?}", keypair_path))?;

    let to_pubkey = Pubkey::from_str(&args.to).with_context(|| "Invalid destination address")?;

    println!("{} Initiating transfer", style("💸").bold());
    println!(
        "{} From: {}",
        style("📤").bold(),
        style(from_keypair.pubkey()).yellow()
    );

    println!("{} To: {}", style("📥").bold(), style(&args.to).yellow());
    println!(
        "{} Amount: {} SOL",
        style("💰").bold(),
        style(args.amount).green()
    );

    let recent_blockhash = client.get_latest_blockhash().await?;

    // 创建计算预算指令
    let compute_unit_price = 500000; // 设置计算单元价格为 0.5 lamports
    let compute_unit_instruction =
        ComputeBudgetInstruction::set_compute_unit_price(compute_unit_price);

    // 创建转账指令
    let transfer_instruction = system_instruction::transfer(
        &from_keypair.pubkey(),
        &to_pubkey,
        sol_to_lamports(args.amount),
    );

    let transaction = Transaction::new_signed_with_payer(
        &[
            compute_unit_instruction, // 首先设置计算预算
            transfer_instruction,     // 然后执行转账
        ],
        Some(&from_keypair.pubkey()),
        &[&from_keypair],
        recent_blockhash,
    );

    print!("{} Sending transaction", style("⏳").bold());
    let term = Term::stdout();
    let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    for i in 0..10 {
        term.clear_line()?;
        term.write_str(&format!(
            "{} Sending transaction {}",
            style("⏳").bold(),
            spinner[i % spinner.len()]
        ))?;
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    term.clear_line()?;

    let signature = client
        .send_and_confirm_transaction(&transaction)
        .await
        .with_context(|| "Failed to send transaction")?;

    println!("{} Transaction successful!", style("✔").green().bold());
    println!(
        "{} Signature: {}",
        style("🔑").bold(),
        style(signature).cyan()
    );

    Ok(())
}
