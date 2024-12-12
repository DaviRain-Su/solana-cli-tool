use {
    solana_client::nonblocking::pubsub_client::PubsubClient,
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature},
    tokio::sync::broadcast,
    anyhow::{Result, anyhow},
    serde::{Deserialize, Serialize},
    futures::StreamExt,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountUpdate {
    Changed {
        slot: u64,
        lamports: u64,
        owner: String,
        data: Vec<u8>,
    },
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionUpdate {
    Received {
        slot: u64,
        signature: String,
    },
    StatusUpdate {
        slot: u64,
        signature: String,
        err: Option<String>,
    },
    Error(String),
}

/// WebSocketMonitor handles real-time monitoring of wallet activities and transaction statuses
/// using Solana's WebSocket PubSub interface.
pub struct WebSocketMonitor {
    client: PubsubClient,
    commitment: CommitmentConfig,
}

impl WebSocketMonitor {
    /// Creates a new WebSocketMonitor instance with the specified WebSocket URL
    pub async fn new(websocket_url: &str) -> Result<Self> {
        let client = PubsubClient::new(websocket_url).await?;
        Ok(Self {
            client,
            commitment: CommitmentConfig::finalized(),
        })
    }

    /// Returns the current commitment configuration
    pub fn commitment(&self) -> CommitmentConfig {
        self.commitment
    }

    /// Sets a new commitment configuration
    pub fn set_commitment(&mut self, commitment: CommitmentConfig) {
        self.commitment = commitment;
    }

    /// Subscribes to account updates for the specified public key
    pub async fn monitor_account(&self, pubkey: &Pubkey) -> Result<broadcast::Receiver<AccountUpdate>> {
        let (subscription, mut receiver) = self.client
            .account_subscribe(
                pubkey,
                Some(self.commitment),
            )
            .await?;

        let (tx, rx) = broadcast::channel(100);

        tokio::spawn(async move {
            while let Some(response) = receiver.next().await {
                match response {
                    Ok(account) => {
                        let update = AccountUpdate::Changed {
                            slot: account.context.slot,
                            lamports: account.value.lamports,
                            owner: account.value.owner.to_string(),
                            data: account.value.data,
                        };
                        if tx.send(update).is_err() {
                            break;
                        }
                    }
                    Err(err) => {
                        let _ = tx.send(AccountUpdate::Error(err.to_string()));
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }

    /// Subscribes to transaction signature updates
    pub async fn monitor_signature(&self, signature: &Signature) -> Result<broadcast::Receiver<TransactionUpdate>> {
        let config = solana_client::rpc_config::RpcSignatureSubscribeConfig {
            commitment: Some(self.commitment),
            enable_received_notification: Some(true),
        };

        let (subscription, mut receiver) = self.client
            .signature_subscribe(
                signature,
                Some(config),
            )
            .await?;

        let (tx, rx) = broadcast::channel(100);
        let signature_str = signature.to_string();

        tokio::spawn(async move {
            while let Some(response) = receiver.next().await {
                match response {
                    Ok(sig_status) => {
                        let update = match sig_status.value.as_str() {
                            "receivedSignature" => TransactionUpdate::Received {
                                slot: sig_status.context.slot,
                                signature: signature_str.clone(),
                            },
                            _ => TransactionUpdate::StatusUpdate {
                                slot: sig_status.context.slot,
                                signature: signature_str.clone(),
                                err: sig_status.value.err.map(|e| e.to_string()),
                            },
                        };
                        if tx.send(update).is_err() {
                            break;
                        }
                    }
                    Err(err) => {
                        let _ = tx.send(TransactionUpdate::Error(err.to_string()));
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }
}
