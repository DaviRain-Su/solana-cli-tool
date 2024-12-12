use {
    crate::{
        config::get_rpc_client,
        utils::lamports_to_sol,
    },
    anyhow::Result,
    chrono::{DateTime, Local, Utc},
    clap::Parser,
    console::{style, Emoji},
    serde::{Deserialize, Serialize},
    solana_sdk::{pubkey::Pubkey, signature::Signature},
    solana_transaction_status_client_types::UiTransactionEncoding,
    std::{collections::HashMap, str::FromStr, sync::Arc, time::Duration},
    tokio::sync::Mutex,
    tokio::time::Instant,
    tokio::{sync::broadcast, time::sleep},
};

#[derive(Parser, Debug)]
pub struct PollMonitorArgs {
    /// Wallet addresses to monitor
    #[clap(long)]
    pub addresses: Vec<String>,
    /// Polling interval in seconds
    #[clap(long, default_value = "10")]
    pub interval: u64,
}

#[derive(Parser, Debug)]
pub struct WebSocketMonitorArgs {
    /// Wallet address to monitor
    #[clap(long)]
    pub address: String,
    /// Enable transaction monitoring
    #[clap(long, default_value = "true")]
    pub monitor_transactions: bool,
}

#[derive(Parser, Debug)]
pub enum MonitorCommand {
    /// Monitor wallet using traditional polling method
    Poll(PollMonitorArgs),
    /// Monitor wallet using WebSocket for real-time updates
    WebSocket(WebSocketMonitorArgs),
}

pub async fn handle_monitor(cmd: MonitorCommand) -> Result<()> {
    match cmd {
        MonitorCommand::Poll(args) => monitor_wallet_poll(args).await,
        MonitorCommand::WebSocket(args) => crate::monitor::websocket_monitor::monitor_wallet_websocket(args).await,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitorEvent {
    BalanceChange {
        address: String,
        old_balance: f64,
        new_balance: f64,
        timestamp: String,
    },
    NewTransaction {
        address: String,
        signature: String,
        timestamp: String,
        status: String,
    },
    TokenBalanceChange {
        address: String,
        token_address: String,
        old_balance: f64,
        new_balance: f64,
        timestamp: String,
    },
}

#[derive(Clone)]
pub struct Monitor {
    balance_cache: Arc<Mutex<HashMap<String, f64>>>,
    token_balance_cache: Arc<Mutex<HashMap<(String, String), f64>>>,
    tx_signature_cache: Arc<Mutex<HashMap<String, Vec<Signature>>>>,
    event_sender: broadcast::Sender<MonitorEvent>,
}

impl Monitor {
    pub fn new() -> (Self, broadcast::Receiver<MonitorEvent>) {
        let (tx, rx) = broadcast::channel(100);

        (
            Self {
                balance_cache: Arc::new(Mutex::new(HashMap::new())),
                token_balance_cache: Arc::new(Mutex::new(HashMap::new())),
                tx_signature_cache: Arc::new(Mutex::new(HashMap::new())),
                event_sender: tx,
            },
            rx,
        )
    }

    // 监控 SOL 余额变化
    async fn monitor_balance(&self, address: &str) -> Result<()> {
        let pubkey = Pubkey::from_str(address)?;
        let rpc_client = get_rpc_client()?;
        let new_balance = rpc_client.get_balance(&pubkey).await? as f64 / 1e9;

        let mut cache = self.balance_cache.lock().await;
        let old_balance = *cache.get(address).unwrap_or(&0.0);

        if (new_balance - old_balance).abs() > 0.000001 {
            // 考虑浮点数精度
            let event = MonitorEvent::BalanceChange {
                address: address.to_string(),
                old_balance,
                new_balance,
                timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            };

            self.event_sender.send(event)?;
            cache.insert(address.to_string(), new_balance);
        }

        Ok(())
    }

    async fn monitor_transactions(&self, address: &str) -> Result<()> {
        let pubkey = Pubkey::from_str(address)?;
        let rpc_client = get_rpc_client()?;

        // 获取最新的签名
        let signatures = rpc_client.get_signatures_for_address(&pubkey).await?;

        for sig_info in signatures.iter().take(10) {
            let signature = Signature::from_str(&sig_info.signature)?;

            let mut cache = self.tx_signature_cache.lock().await;
            let known_signatures = cache.entry(address.to_string()).or_insert_with(Vec::new);

            // 检查是否是新交易
            if !known_signatures.contains(&signature) {
                // 获取交易详情
                if let Ok(tx) = rpc_client
                    .get_transaction(&signature, UiTransactionEncoding::Json)
                    .await
                {
                    let status = if sig_info.err.is_some() {
                        "Failed".to_string()
                    } else {
                        "Success".to_string()
                    };

                    let timestamp = if let Some(block_time) = tx.block_time {
                        DateTime::<Utc>::from_timestamp(block_time, 0)
                            .unwrap_or_else(|| Utc::now())
                            .with_timezone(&Local)
                            .format("%Y-%m-%d %H:%M:%S")
                            .to_string()
                    } else {
                        Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
                    };

                    let event = MonitorEvent::NewTransaction {
                        address: address.to_string(),
                        signature: signature.to_string(),
                        timestamp,
                        status,
                    };

                    self.event_sender.send(event)?;
                    known_signatures.push(signature);

                    // 只保留最近的50个签名
                    if known_signatures.len() > 50 {
                        known_signatures.drain(0..known_signatures.len() - 50);
                    }
                }
            }
        }

        // 添加延迟以避免过于频繁的请求
        tokio::time::sleep(Duration::from_secs(1)).await;
        Ok(())
    }

async fn monitor_token_balance(&self, wallet_address: &str, token_address: &str) -> Result<()> {
    static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍 ", "");
    static WALLET: Emoji<'_, '_> = Emoji("👛 ", "");
    static TOKEN: Emoji<'_, '_> = Emoji("🪙 ", "");
    static WARNING: Emoji<'_, '_> = Emoji("⚠️ ", "");
    static CHECK: Emoji<'_, '_> = Emoji("✅ ", "");
    static ALERT: Emoji<'_, '_> = Emoji("🔔 ", "");

    println!("{}", style("Token Balance Monitor").bold().bright().cyan());
    println!(
        "{}{}: {}",
        WALLET,
        style("Wallet").dim(),
        style(wallet_address).green()
    );
    println!(
        "{}{}: {}",
        TOKEN,
        style("Token").dim(),
        style(token_address).green()
    );
    println!();

    let rpc_client = match get_rpc_client() {
        Ok(client) => client,
        Err(e) => {
            println!(
                "{} {}",
                style("ERROR:").red().bold(),
                style(format!("Failed to get RPC client: {:?}", e)).red()
            );
            return Err(e);
        }
    };

    let wallet_pubkey = match Pubkey::from_str(wallet_address) {
        Ok(pubkey) => pubkey,
        Err(e) => {
            println!(
                "{} {}",
                style("ERROR:").red().bold(),
                style(format!(
                    "Invalid wallet address {}: {:?}",
                    wallet_address, e
                ))
                .red()
            );
            return Err(e.into());
        }
    };

    let token_pubkey = match Pubkey::from_str(token_address) {
        Ok(pubkey) => pubkey,
        Err(e) => {
            println!(
                "{} {}",
                style("ERROR:").red().bold(),
                style(format!("Invalid token address {}: {:?}", token_address, e)).red()
            );
            return Err(e.into());
        }
    };

    let mut last_print_time = Instant::now();
    let print_interval = Duration::from_secs(60);

    loop {
        let filter = solana_client::rpc_request::TokenAccountsFilter::Mint(token_pubkey);

        match rpc_client
            .get_token_accounts_by_owner(&wallet_pubkey, filter)
            .await
        {
            Ok(accounts) => {
                if accounts.is_empty() {
                    if last_print_time.elapsed() >= print_interval {
                        println!(
                            "{}{}",
                            WARNING,
                            style("No token accounts found. Wallet might not hold this token.")
                                .yellow()
                        );
                        last_print_time = Instant::now();
                    }
                    continue;
                }

                for account in accounts {
                    match rpc_client
                        .get_token_account_balance(&Pubkey::from_str(&account.pubkey)?)
                        .await
                    {
                        Ok(balance) => {
                            let new_balance = balance.ui_amount.unwrap_or_default();
                            let mut cache = self.token_balance_cache.lock().await;
                            let key = (wallet_address.to_string(), token_address.to_string());
                            let old_balance = *cache.get(&key).unwrap_or(&0.0);

                            if (new_balance - old_balance).abs() > 0.000001 {
                                println!(
                                    "\n{} {}",
                                    ALERT,
                                    style("Balance Change Detected").bold().yellow()
                                );
                                println!("{}", style("─".repeat(50)).dim());
                                println!(
                                    "{}{}: {}",
                                    LOOKING_GLASS,
                                    style("Account").dim(),
                                    style(&account.pubkey).cyan()
                                );

                                // 计算余额变化
                                let change = new_balance - old_balance;
                                let change_str = format!("{:+.6}", change);
                                let change_style = if change > 0.0 {
                                    style(change_str).green()
                                } else {
                                    style(change_str).red()
                                };

                                println!(
                                    "Old Balance: {}",
                                    style(format!("{:.6}", old_balance)).red()
                                );
                                println!(
                                    "New Balance: {}",
                                    style(format!("{:.6}", new_balance)).green()
                                );
                                println!("Change:      {}", change_style);
                                println!("{}", style("─".repeat(50)).dim());

                                let event = MonitorEvent::TokenBalanceChange {
                                    address: wallet_address.to_string(),
                                    token_address: token_address.to_string(),
                                    old_balance,
                                    new_balance,
                                    timestamp: Local::now()
                                        .format("%Y-%m-%d %H:%M:%S")
                                        .to_string(),
                                };

                                if let Err(e) = self.event_sender.send(event) {
                                    println!(
                                        "{} {}",
                                        style("ERROR:").red().bold(),
                                        style(format!("Failed to send event: {:?}", e)).red()
                                    );
                                }

                                cache.insert(key, new_balance);
                            } else if last_print_time.elapsed() >= print_interval {
                                println!(
                                    "{} {} {} {}",
                                    CHECK,
                                    style("Balance").dim(),
                                    style(format!("{:.6}", new_balance)).cyan(),
                                    style(Local::now().format("%H:%M:%S")).dim()
                                );
                                last_print_time = Instant::now();
                            }
                        }
                        Err(e) => {
                            println!(
                                "{} {}",
                                style("ERROR:").red().bold(),
                                style(format!(
                                    "Error getting balance for account {}: {:?}",
                                    account.pubkey, e
                                ))
                                .red()
                            );
                        }
                    }
                }
            }
            Err(e) => {
                println!(
                    "{} {}",
                    style("ERROR:").red().bold(),
                    style(format!("Error getting token accounts: {:?}", e)).red()
                );
            }
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

/// Monitor wallet activities using polling method
async fn monitor_wallet_poll(args: PollMonitorArgs) -> Result<()> {
    let (monitor, mut rx) = Monitor::new();

    // Start balance monitoring
    for address in args.addresses.iter() {
        let monitor = monitor.clone();
        let address = address.to_string();
        tokio::spawn(async move {
            loop {
                if let Err(e) = monitor.monitor_balance(&address).await {
                    eprintln!("Error monitoring balance: {}", e);
                }
                sleep(Duration::from_secs(args.interval)).await;
            }
        });
    }

    // Start transaction monitoring
    for address in args.addresses {
        let monitor = monitor.clone();
        let address = address.to_string();
        tokio::spawn(async move {
            loop {
                if let Err(e) = monitor.monitor_transactions(&address).await {
                    eprintln!("Error monitoring transactions: {}", e);
                }
                sleep(Duration::from_secs(args.interval)).await;
            }
        });
    }

    // Handle events
    while let Ok(event) = rx.recv().await {
        match event {
            MonitorEvent::BalanceChange {
                address,
                old_balance,
                new_balance,
                timestamp,
            } => {
                println!(
                    "💰 Balance change for {}: {} -> {} SOL ({})",
                    address, old_balance, new_balance, timestamp
                );
            }
            MonitorEvent::NewTransaction {
                address,
                signature,
                timestamp,
                status,
            } => {
                println!(
                    "📝 New transaction for {}: {} ({}) [{}]",
                    address, signature, status, timestamp
                );
            }
            _ => {}
        }
    }

    Ok(())
}
