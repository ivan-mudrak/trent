use crate::types::{
    error::{ErrorKind, Result},
    tx::{Tx, TxId},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Clone, Debug)]
pub struct Account {
    pub state: Arc<Mutex<AccountState>>,
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
            state: Arc::new(Mutex::new(AccountState::default())),
        }
    }

    pub fn try_settle_tx(&mut self, tx: Tx) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if state.locked {
            return Err(ErrorKind::AccountBlocked.into());
        }

        match tx {
            Tx::Deposit { id, amount } => {
                state.deposits.insert(id, amount);
                state.available += amount;
                state.total += amount;
            }
            Tx::Withdrawal { amount, .. } => {
                if state.available >= amount {
                    state.available -= amount;
                    state.total -= amount;
                } else {
                    return Err(ErrorKind::NotEnoughFunds.into());
                }
            }
            Tx::Dispute { id } => {
                if state.active_disputes.contains_key(&id) {
                    return Err(ErrorKind::TransactionAlreadyUnderDispute.into());
                } else {
                    if let Some(amount) = state.deposits.get(&id).copied() {
                        state.active_disputes.insert(id, amount);
                        state.available -= amount;
                        state.held += amount;
                    } else {
                        return Err(ErrorKind::NoneExistingTxRef(id).into());
                    }
                }
            }
            Tx::Resolve { id } => {
                if let Some(amount) = state.active_disputes.remove(&id) {
                    state.available += amount;
                    state.held -= amount;
                } else {
                    return Err(ErrorKind::NoneExistingTxRef(id).into());
                }
            }
            Tx::Chargeback { id } => {
                if let Some(amount) = state.active_disputes.remove(&id) {
                    state.total -= amount;
                    state.held -= amount;
                    state.locked = true;
                } else {
                    return Err(ErrorKind::NoneExistingTxRef(id).into());
                }
            }
        };
        Ok(())
    }
}
