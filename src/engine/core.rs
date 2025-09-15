use crate::types::{
    client::{ClientId, ClientTx, Clients},
    error::Result,
};
use std::collections::HashMap;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio::task::JoinHandle;

pub struct TxEngine {
    handles: Vec<JoinHandle<Clients>>,
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
            clients: Clients(HashMap::new()),
        }
    }

    pub fn multi_threaded() -> TxEngine {
        let num_threads = num_cpus::get();

        if num_threads > 1 {
            let mut senders = Vec::<Sender<Message>>::with_capacity(num_threads);
            let mut handles = Vec::<JoinHandle<Clients>>::with_capacity(num_threads);
            let clients = Clients(HashMap::new());

            for _ in 0..num_threads {
                let (tx, rx) = channel(1024);
                senders.push(tx);
                handles.push(Self::spawn_worker(rx));
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

    pub async fn process(&mut self, txs: Vec<ClientTx>) -> Result<()> {
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

    fn spawn_worker(mut rx: Receiver<Message>) -> JoinHandle<Clients> {
        tokio::task::spawn(async move {
            let mut clients = Clients(HashMap::new());
            loop {
                match rx.recv().await {
                    Some(Message::Process(tx)) => clients.process_tx(tx),
                    Some(Message::Shutdown) => break,
                    None => break,
                }
            }
            clients
        })
    }

    pub async fn flush(mut self) -> Result<Box<Clients>> {
        for sender in self.senders {
            let _ = sender.send(Message::Shutdown).await;
        }

        for handle in self.handles {
            match handle.await {
                Ok(batch) => self.clients.0.extend(batch.0),
                Err(_) => {
                    // TODO: log error
                }
            }
        }
        Ok(Box::new(self.clients))
    }
}
