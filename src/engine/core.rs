use crate::types::client::ClientId;
use crate::types::{
    client::{ClientTx, Clients},
    error::Result,
};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

pub struct TxEngine {
    handles: Vec<JoinHandle<()>>,
    senders: Vec<Sender<Message>>,
    clients: Clients,
}
pub enum Message {
    Process(ClientTx),
    Shutdown,
}

impl TxEngine {
    pub fn single_threaded() -> TxEngine {
        TxEngine {
            handles: vec![],
            senders: vec![],
            clients: Clients(Arc::new(DashMap::new())),
        }
    }

    pub fn multi_threaded() -> TxEngine {
        let num_threads = num_cpus::get();

        if num_threads > 1 {
            let mut senders = Vec::<Sender<Message>>::with_capacity(num_threads);
            let mut handles = Vec::<JoinHandle<()>>::with_capacity(num_threads);
            let clients = Clients(Arc::new(DashMap::new()));

            for _ in 0..num_threads {
                let (tx, rx) = tokio::sync::mpsc::channel(1024);
                senders.push(tx);
                handles.push(Self::spawn_worker(clients.clone(), rx));
            }

            TxEngine {
                handles,
                senders,
                clients,
            }
        } else {
            Self::single_threaded()
        }
    }

    pub async fn process(&self, txs: Vec<ClientTx>) -> Result<()> {
        if txs.is_empty() {
            return Ok(());
        };

        if self.senders.is_empty() {
            for tx in txs {
                self.clients.process_tx(tx);
            }
        } else {
            for tx in txs {
                // IMPORTANT: client pinned to a worker
                let shard = Self::client_shard(tx.client_id, self.senders.len());
                let _ = self.senders[shard].send(Message::Process(tx)).await;
            }
        }

        Ok(())
    }

    fn client_shard(client_id: ClientId, num_shards: usize) -> usize {
        // TODO: improve distribution
        (client_id as usize) % num_shards
    }

    fn spawn_worker(clients: Clients, mut rx: Receiver<Message>) -> JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    Message::Process(tx) => clients.process_tx(tx),
                    Message::Shutdown => break,
                }
            }
        })
    }

    pub async fn flush(self) -> Result<Box<Clients>> {
        for sender in self.senders {
            let _ = sender.send(Message::Shutdown).await;
        }
        for handle in self.handles {
            let _ = handle.await;
        }
        Ok(Box::new(self.clients))
    }
}
