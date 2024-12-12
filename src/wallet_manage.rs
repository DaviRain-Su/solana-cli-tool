use clap::Parser;

pub mod account_create_timestamp;
pub mod balance;
pub mod display_private_key;
pub mod list_wallets;
pub mod new_wallet;
pub mod recover;
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
    /// Get account create timestamp
    AccountCreateTimestamp(account_create_timestamp::AccountCreateTimestampArgs),
    /// list all wallets
    ListWallets,
    // Display Wallet Private Key
    DisplayWalletPrivateKey(display_private_key::DisplayPrivateKeyArgs),
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
        WalletMange::AccountCreateTimestamp(args) => {
            //println!("Get account create timestamp for address: {}", address);
            account_create_timestamp::handle_account_create_timestamp(args).await?;
            Ok(())
        }
        WalletMange::ListWallets => {
            println!("List all wallets");
            list_wallets::list_all_wallets()?;
            Ok(())
        }
        WalletMange::DisplayWalletPrivateKey(args) => {
            display_private_key::display_private_key(args)
        }
        WalletMange::Config => crate::config::show_config(),
    }
}
