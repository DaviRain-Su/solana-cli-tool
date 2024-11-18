use clap::Parser;

pub mod new_wallet;
pub mod recover_private_key;

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
    Transfer {
        #[clap(short, long)]
        from: String,
        #[clap(short, long)]
        to: String,
        #[clap(short, long)]
        amount: f64,
    },
    /// Transfer SPL token
    TransferToken {
        #[clap(short, long)]
        from: String,
        #[clap(short, long)]
        to: String,
        #[clap(short, long)]
        amount: f64,
        #[clap(short, long)]
        token: String,
    },
}

pub async fn handle_wallet_manage(wallet_manage: &WalletMange) -> anyhow::Result<()> {
    match wallet_manage {
        WalletMange::NewWallet(args) => new_wallet::create_new_wallet(args),
        WalletMange::RecoverPrivateKey { private_key } => {
            println!("Recover wallet from private key: {}", private_key);
            recover_private_key::recover_from_private_key(private_key)
        }
        WalletMange::RecoverWallet { mnemonic } => {
            println!("Recover wallet from mnemonic: {}", mnemonic);
            Ok(())
        }
        WalletMange::Balance { address } => {
            println!("Check wallet balance: {:?}", address);
            Ok(())
        }
        WalletMange::Transfer { from, to, amount } => {
            println!("Transfer SOL from {} to {} amount: {}", from, to, amount);
            Ok(())
        }
        WalletMange::TransferToken {
            from,
            to,
            amount,
            token,
        } => {
            println!(
                "Transfer SPL token from {} to {} amount: {} token: {}",
                from, to, amount, token
            );
            Ok(())
        }
    }
}
