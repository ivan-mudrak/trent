use crate::types::{account::Account, tx::Tx};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
pub struct Clients(pub HashMap<ClientId, Account>);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ClientAccount {
    pub client: ClientId,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

impl Clients {
    pub fn process_tx(&mut self, client_tx: ClientTx) -> () {
        let acc = self.0.entry(client_tx.client_id).or_insert(Account::new());
        match acc.try_settle_tx(client_tx.tx) {
            Ok(_) => {}
            Err(_) => {
                // TODO: log errors
            }
        }
    }

    pub fn write_to_csv<W: std::io::Write>(self, wrt: W) -> Result<(), csv::Error> {
        let mut writer = csv::Writer::from_writer(wrt);
        let clients = self.0;
        for (client, acc) in clients {
            let row = ClientAccount {
                client,
                available: acc.state.available,
                held: acc.state.held,
                total: acc.state.total,
                locked: acc.state.locked,
            };
            writer.serialize(row)?;
        }

        writer.flush()?;
        Ok(())
    }
}
