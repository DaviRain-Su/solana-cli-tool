use clap::Parser;

pub mod balance;
pub mod new_wallet;
pub mod recover_private_key;
pub mod transfer;

#[derive(Parser, Debug)]
pub enum WalletMange {
    /// Create a new wallet
    NewWallet(new_wallet::NewWalletArgs),
    /// recover wallet from mnemonic
    RecoverWallet {
        #[clap(short, long)]
        mnemonic: String,
    },
    /// recover wallet from base58 encoded private key
    RecoverPrivateKey(recover_private_key::RecoverPrivateKeyArgs),
    /// Check wallet balance
    Balance(balance::BalanceArgs),
    /// Transfer SOL
    Transfer(transfer::TransferArgs),
    /// Transfer SPL token
    TransferToken {
        /// Sender wallet address, if it is not provided, the default wallet will be used
        #[clap(short, long)]
        from: Option<String>,
        #[clap(short, long)]
        to: String,
        #[clap(short, long)]
        amount: f64,
        #[clap(short, long)]
        token: String,
    },
    /// Show current configuration
    Config,
}

pub async fn handle_wallet_manage(wallet_manage: &WalletMange) -> anyhow::Result<()> {
    match wallet_manage {
        WalletMange::NewWallet(args) => new_wallet::create_new_wallet(args),
        WalletMange::RecoverPrivateKey(args) => recover_private_key::recover_from_private_key(args),
        WalletMange::RecoverWallet { mnemonic } => {
            println!("Recover wallet from mnemonic: {}", mnemonic);
            Ok(())
        }
        WalletMange::Balance(args) => balance::display_balance(args).await,
        WalletMange::Transfer(arg) => transfer::transfer_sol(arg).await,
        WalletMange::TransferToken {
            from,
            to,
            amount,
            token,
        } => {
            println!(
                "Transfer SPL token from {:?} to {} amount: {} token: {}",
                from, to, amount, token
            );
            Ok(())
        }
        WalletMange::Config => crate::config::show_config(),
    }
}
