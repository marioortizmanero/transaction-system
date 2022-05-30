use crate::currency::PreciseCurrency;

use std::collections::HashMap;

use serde::{de, Deserialize, Serialize};

/// We need 4 digits of precision by default
pub type Currency = PreciseCurrency<4>;

/// I tried to optimize this by saving them into a fixed size array, but it was
/// too large for the stack and it had to be boxed. Even in that case, it was
/// slightly slower, so I ended up keeping the HashMap. Nevertheless, with more
/// time this approach could still be viable, i.e., after reducing the size of
/// the client, or with a number of clients large enough.
pub type AllBalances = HashMap<u16, Balance>;

/// The string fields are case insensitive. This is simpler than implementing
/// `Deserialize` and is only needed once anyway.
fn case_insensitive_transaction_types<'de, D>(deserializer: D) -> Result<TransactionType, D::Error>
where
    D: de::Deserializer<'de>,
{
    match String::deserialize(deserializer)?.to_lowercase().as_str() {
        "deposit" => Ok(TransactionType::Deposit),
        "withdrawal" => Ok(TransactionType::Withdrawal),
        "dispute" => Ok(TransactionType::Dispute),
        "resolve" => Ok(TransactionType::Resolve),
        "chargeback" => Ok(TransactionType::Chargeback),
        other => Err(de::Error::invalid_value(
            de::Unexpected::Str(other),
            &"Must be one of (deposit, withdrawal, dispute, resolve, chargeback)",
        )),
    }
}

/// The transaction types supported for this implementation.
///
/// Another way to save the transactions would be with enum structs (i.e.
/// `Deposit { client: u16, .. }`), since the amount is only necessary for
/// deposits and withdrawals. However, the serialization was easier this way,
/// and this only requires a couple controlled `unwrap`s.
#[derive(Debug, Deserialize, Eq, PartialEq)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

/// The input format, which is deserialized from the CSV thanks to serde. It's
/// important to use `#[serde(default)]` when possible so that it's possible to
/// be more flexible about the input fields by making them optional.
#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct Transaction {
    #[serde(
        rename = "type",
        deserialize_with = "case_insensitive_transaction_types"
    )]
    pub _type: TransactionType,
    #[serde(default)]
    pub client: u16,
    #[serde(default)]
    pub tx: u32,
    #[serde(default)]
    pub amount: Option<Currency>,
}

/// Keeping track of what transactions have been disputed and in which ways.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DisputeState {
    /// Waiting for resolution
    Waiting,
    /// Already was resolved
    Resolved,
}

/// The output format, which is also written to CSV thanks to serde.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[serde(default)]
pub struct Balance {
    pub client: u16,
    pub available: Currency,
    pub held: Currency,
    pub total: Currency,
    pub locked: bool,

    /// Saving the transactions for a user to check later in case of a dispute.
    #[serde(skip)]
    pub transactions: HashMap<u32, Transaction>,
    /// If the transaction isn't in the map, then it isn't disputed.
    #[serde(skip)]
    pub disputes: HashMap<u32, DisputeState>,
}

/// No need to compare the transactions
impl PartialEq for Balance {
    fn eq(&self, other: &Self) -> bool {
        self.client.eq(&other.client)
            && self.available.eq(&other.available)
            && self.held.eq(&other.held)
            && self.total.eq(&other.total)
            && self.locked.eq(&other.locked)
    }
}
impl Eq for Balance {}
