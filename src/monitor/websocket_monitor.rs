use {
    crate::{
        config::get_config,
        websocket::{WebSocketMonitor, AccountUpdate, TransactionUpdate},
        utils::lamports_to_sol,
    },
    anyhow::Result,
    solana_sdk::{pubkey::Pubkey, signature::Signature},
    std::str::FromStr,
    tokio::time::sleep,
    std::time::Duration,
    solana_client::rpc_client::RpcClient,
};

#[derive(Debug, Clone)]
pub struct WebSocketMonitorArgs {
    pub address: String,
    pub monitor_transactions: bool,
}

/// Monitors a wallet's activities in real-time using WebSocket connection
pub async fn monitor_wallet_websocket(args: WebSocketMonitorArgs) -> Result<()> {
    let config = get_config()?;
    let pubkey = Pubkey::from_str(&args.address)?;
    let rpc_client = RpcClient::new(config.json_rpc_url.clone());

    println!("🔍 Starting real-time monitoring for wallet: {}", args.address);
    println!("📡 Connecting to WebSocket...");

    let monitor = WebSocketMonitor::new(&config.websocket_url).await?;
    let account_receiver = monitor.monitor_account(&pubkey).await?;

    println!("✅ WebSocket connection established");
    println!("💫 Monitoring account balance changes...");
    if args.monitor_transactions {
        println!("📝 Transaction monitoring enabled");
    }

    // Start balance monitoring
    tokio::spawn({
        async move {
            while let Ok(update) = account_receiver.recv().await {
                match update {
                    AccountUpdate::Changed { slot, lamports, owner, .. } => {
                        println!("\n💰 Balance update at slot {}", slot);
                        println!("   Balance: {} SOL", lamports_to_sol(lamports));
                        println!("   Owner: {}", owner);
                    }
                    AccountUpdate::Error(err) => {
                        eprintln!("❌ Error monitoring account: {}", err);
                        break;
                    }
                }
            }
        }
    });

    // Start transaction monitoring if enabled
    if args.monitor_transactions {
        let monitor_clone = monitor;
        let pubkey_clone = pubkey;
        tokio::spawn(async move {
            let mut last_signature = None;

            loop {
                // Get recent signatures
                if let Ok(signatures) = rpc_client
                    .get_signatures_for_address(&pubkey_clone)
                    .await
                {
                    if let Some(sig_info) = signatures.first() {
                        let signature = Signature::from_str(&sig_info.signature)?;

                        // Only monitor new signatures
                        if Some(&signature) != last_signature.as_ref() {
                            last_signature = Some(signature);

                            if let Ok(tx_receiver) = monitor_clone.monitor_signature(&signature).await {
                                while let Ok(update) = tx_receiver.recv().await {
                                    match update {
                                        TransactionUpdate::Received { slot, signature } => {
                                            println!("\n📨 Transaction received at slot {}", slot);
                                            println!("   Signature: {}", signature);
                                        }
                                        TransactionUpdate::StatusUpdate { slot, signature, err } => {
                                            let status = if let Some(err) = err {
                                                format!("❌ Failed: {}", err)
                                            } else {
                                                "✅ Confirmed".to_string()
                                            };
                                            println!("\n📝 Transaction status update at slot {}", slot);
                                            println!("   Signature: {}", signature);
                                            println!("   Status: {}", status);
                                        }
                                        TransactionUpdate::Error(err) => {
                                            eprintln!("❌ Error monitoring transaction: {}", err);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                sleep(Duration::from_secs(2)).await;
            }
        });
    }

    // Keep the main task running
    loop {
        sleep(Duration::from_secs(1)).await;
    }

    #[allow(unreachable_code)]
    Ok(())
}
