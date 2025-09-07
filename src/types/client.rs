use crate::types::{account::Account, tx::Tx};
use dashmap::DashMap;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub type ClientId = u16;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ClientTx {
    #[serde(rename = "client")]
    pub client_id: ClientId,
    #[serde(flatten)]
    pub tx: Tx,
}

#[derive(Clone, Debug)]
pub struct Clients(pub Arc<DashMap<ClientId, Account>>);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ClientAccount {
    pub client: ClientId,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

impl Clients {
    pub fn process_tx(&self, client_tx: ClientTx) -> () {
        let mut acc = self.0.entry(client_tx.client_id).or_insert(Account::new());
        match acc.try_settle_tx(client_tx.tx) {
            Ok(_) => {}
            Err(_) => {
                // TODO: log errors
            }
        }
    }

    pub fn write_to_csv<W: std::io::Write>(self, wrt: W) -> Result<(), csv::Error> {
        let mut writer = csv::Writer::from_writer(wrt);
        let clients = Arc::try_unwrap(self.0).unwrap();
        for (client, acc) in clients {
            let state = acc
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            let row = ClientAccount {
                client,
                available: state.available,
                held: state.held,
                total: state.total,
                locked: state.locked,
            };
            writer.serialize(row)?;
        }

        writer.flush()?;
        Ok(())
    }
}
