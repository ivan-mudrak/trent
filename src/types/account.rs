use crate::types::{
    error::{ErrorKind, Result},
    tx::{Tx, TxId},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Account {
    pub state: AccountState,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AccountState {
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
    pub deposits: HashMap<TxId, Decimal>,
    pub active_disputes: HashMap<TxId, Decimal>,
}

impl Account {
    pub fn new() -> Self {
        Account {
            state: AccountState::default(),
        }
    }

    pub fn try_settle_tx(&mut self, tx: Tx) -> Result<()> {
        if self.state.locked {
            return Err(ErrorKind::AccountBlocked.into());
        }

        match tx {
            Tx::Deposit { id, amount } => {
                self.state.deposits.insert(id, amount);
                self.state.available += amount;
                self.state.total += amount;
            }
            Tx::Withdrawal { amount, .. } => {
                if self.state.available >= amount {
                    self.state.available -= amount;
                    self.state.total -= amount;
                } else {
                    return Err(ErrorKind::NotEnoughFunds.into());
                }
            }
            Tx::Dispute { id } => {
                if self.state.active_disputes.contains_key(&id) {
                    return Err(ErrorKind::TransactionAlreadyUnderDispute.into());
                } else {
                    if let Some(amount) = self.state.deposits.get(&id).copied() {
                        self.state.active_disputes.insert(id, amount);
                        self.state.available -= amount;
                        self.state.held += amount;
                    } else {
                        return Err(ErrorKind::NoneExistingTxRef(id).into());
                    }
                }
            }
            Tx::Resolve { id } => {
                if let Some(amount) = self.state.active_disputes.remove(&id) {
                    self.state.available += amount;
                    self.state.held -= amount;
                } else {
                    return Err(ErrorKind::NoneExistingTxRef(id).into());
                }
            }
            Tx::Chargeback { id } => {
                if let Some(amount) = self.state.active_disputes.remove(&id) {
                    self.state.total -= amount;
                    self.state.held -= amount;
                    self.state.locked = true;
                } else {
                    return Err(ErrorKind::NoneExistingTxRef(id).into());
                }
            }
        };
        Ok(())
    }
}
