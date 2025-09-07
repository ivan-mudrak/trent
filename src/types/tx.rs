use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub type TxId = u32;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum Tx {
    Deposit {
        #[serde(rename = "tx")]
        id: TxId,
        amount: Decimal,
    },
    Withdrawal {
        #[serde(rename = "tx")]
        id: TxId,
        amount: Decimal,
    },
    Dispute {
        #[serde(rename = "tx")]
        id: TxId,
    },
    Resolve {
        #[serde(rename = "tx")]
        id: TxId,
    },
    Chargeback {
        #[serde(rename = "tx")]
        id: TxId,
    },
}
