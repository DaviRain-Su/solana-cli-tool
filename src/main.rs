use clap::Parser;

pub mod config;
pub mod monitor;
pub mod spl_token_manage;
pub mod utils;
pub mod wallet_manage;

#[derive(Parser, Debug)]
#[clap(name = "solana-cli-tool", version, author, about)]
enum Commands {
    /// Manage wallet
    #[command(subcommand)]
    Wallet(wallet_manage::WalletMange),
    /// Spl token manage
    #[command(subcommand)]
    SplToken(spl_token_manage::SplTokenMange),
    /// monitor
    Monitor(monitor::MonitorArgs),
}

impl Commands {
    async fn run(&self) -> anyhow::Result<()> {
        match self {
            Commands::Wallet(wallet_manage) => {
                wallet_manage::handle_wallet_manage(wallet_manage).await
            }
            Commands::SplToken(spl_token_manage) => {
                spl_token_manage::handle_spl_token_manage(spl_token_manage).await
            }
            Commands::Monitor(monitor_args) => monitor::run_monitor(monitor_args).await,
        }
    }
}

#[tokio::main]
async fn main() {
    let cmd = Commands::parse();
    if let Err(e) = cmd.run().await {
        eprintln!("Error: {:?}", e);
    }
}
