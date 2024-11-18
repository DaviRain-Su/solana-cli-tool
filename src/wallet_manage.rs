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
    RecoverPrivateKey {
        #[clap(short, long)]
        private_key: String,
    },
    /// Check wallet balance
    Balance {
        #[clap(short, long)]
        address: Option<String>,
    },
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
        WalletMange::RecoverPrivateKey { private_key } => {
            recover_private_key::recover_from_private_key(private_key)
        }
        WalletMange::RecoverWallet { mnemonic } => {
            println!("Recover wallet from mnemonic: {}", mnemonic);
            Ok(())
        }
        WalletMange::Balance { address } => {
            balance::display_balance(address.as_ref().map(|x| x.as_str())).await
        }
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
