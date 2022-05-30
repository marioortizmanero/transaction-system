pub mod currency;
pub mod model;

use model::{AllBalances, Balance, DisputeState, Transaction, TransactionType};

use std::collections::HashMap;
use std::env;

use anyhow::{anyhow, Result};

impl Balance {
    /// Deposits are the only valid first transaction. But even if the first one
    /// is an invalid withdrawal, it should be saved internally to avoid
    /// duplicate transaction IDs. The rest of the types should still be
    /// ignored, in which case `None` is returned.
    fn from_tx(tx: Transaction) -> Option<Self> {
        match tx {
            Transaction {
                _type: TransactionType::Deposit,
                amount: Some(amount),
                ..
            } => Some(Balance {
                client: tx.client,
                available: amount,
                total: amount,
                transactions: {
                    let mut map = HashMap::with_capacity(1);
                    map.insert(tx.tx, tx);
                    map
                },
                ..Balance::default()
            }),
            Transaction {
                _type: TransactionType::Withdrawal,
                ..
            } => Some(Balance {
                client: tx.client,
                transactions: {
                    let mut map = HashMap::with_capacity(1);
                    map.insert(tx.tx, tx);
                    map
                },
                ..Balance::default()
            }),
            _ => None,
        }
    }

    /// Applies a new transaction of any kind to the balance.
    fn apply_tx(&mut self, tx: Transaction) {
        // Frozen accounts should be ignored
        if self.locked {
            return;
        }

        // The actual transaction engine, implemented as described in the
        // `README.md`.
        match tx._type {
            TransactionType::Deposit => {
                // If the transaction already exists, do nothing
                self.transactions.entry(tx.tx).or_insert_with(|| {
                    let amount = tx.amount.unwrap();
                    self.available += amount;
                    self.total += amount;
                    tx
                });
            }
            TransactionType::Withdrawal => {
                let amount = tx.amount.unwrap();
                // Operation is cancelled if there aren't enough available funds
                if self.available - amount < 0.into() {
                    return;
                }

                // If the transaction already exists, do nothing
                self.transactions.entry(tx.tx).or_insert_with(|| {
                    self.available -= amount;
                    self.total -= amount;
                    tx
                });
            }
            TransactionType::Dispute => {
                // If the transaction doesn't exist, do nothing.
                if let Some(tx) = self.transactions.get(&tx.tx) {
                    // If its entry already exists, i.e., it's already disputed
                    // or resolved, it can't be disputed again.
                    self.disputes.entry(tx.tx).or_insert_with(|| {
                        let amount = tx.amount.unwrap();
                        // No need to do anything for withdrawals
                        if tx._type == TransactionType::Deposit {
                            self.available -= amount;
                            self.held += amount;
                        }
                        DisputeState::Waiting
                    });
                }
            }
            TransactionType::Resolve => {
                // If the transaction doesn't exist, do nothing.
                if let Some(tx) = self.transactions.get(&tx.tx) {
                    // If no entry exists, i.e., it's undisputed, or if it's
                    // already resolved, nothing should happen.
                    match self.disputes.get_mut(&tx.tx) {
                        None | Some(DisputeState::Resolved) => {}
                        Some(ds @ DisputeState::Waiting) => {
                            let amount = tx.amount.unwrap();
                            // No need to do anything for withdrawals
                            if tx._type == TransactionType::Deposit {
                                self.available += amount;
                                self.held -= amount;
                            }
                            *ds = DisputeState::Resolved;
                        }
                    }
                }
            }
            TransactionType::Chargeback => {
                // If the transaction doesn't exist, do nothing.
                if let Some(tx) = self.transactions.get(&tx.tx) {
                    // If no entry exists, i.e., it's undisputed, or if it's
                    // already resolved, nothing should happen.
                    match self.disputes.get_mut(&tx.tx) {
                        None | Some(DisputeState::Resolved) => {}
                        Some(ds @ DisputeState::Waiting) => {
                            let amount = tx.amount.unwrap();
                            match tx._type {
                                TransactionType::Deposit => {
                                    self.held -= amount;
                                    self.total -= amount;
                                }
                                TransactionType::Withdrawal => {
                                    self.available += amount;
                                    self.total += amount;
                                }
                                _ => {}
                            }
                            self.locked = true;
                            // Doesn't really matter here anyway, since its
                            // account is now frozen and no other operations
                            // will be performed.
                            *ds = DisputeState::Resolved;
                        }
                    }
                }
            }
        }
    }
}

/// It's possible that the csv has spacing between fields, so we must enable the
/// trim option.
pub fn init_reader(file: &str) -> csv::Result<csv::Reader<std::fs::File>> {
    csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(file)
}

/// Given an input file, return the final balances.
pub fn process(file: &str) -> Result<AllBalances> {
    let mut balances = AllBalances::new();
    let mut reader = init_reader(file)?;

    for result in reader.deserialize::<Transaction>() {
        // Error resilience: the program tries to continue after finding an
        // erroneous entry.
        match result {
            Ok(entry) => {
                let client = entry.client;
                match balances.get_mut(&client) {
                    // Uninitialized client
                    None => {
                        if let Some(balance) = Balance::from_tx(entry) {
                            balances.insert(client, balance);
                        }
                    }
                    // Previously intialized client
                    Some(ref mut client) => client.apply_tx(entry),
                }
            }
            Err(e) => {
                eprintln!("Failed to read CSV entry: {e}");
            }
        }
    }

    Ok(balances)
}

/// Run the program as configured by the arguments, and write the result to the
/// standard output.
pub fn load() -> Result<()> {
    let file = env::args()
        .nth(1)
        .ok_or_else(|| anyhow!("no transactions filename passed"))?;

    let clients = process(&file)?;

    let mut writer = csv::Writer::from_writer(std::io::stdout());
    for client in clients.values() {
        writer.serialize(client)?;
    }

    Ok(())
}
