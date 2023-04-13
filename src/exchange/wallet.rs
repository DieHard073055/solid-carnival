use crate::exchange::transaction::Transaction;
use chrono::Utc;
use rust_decimal::prelude::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

/**
This code defines a Wallet struct that tracks a user's wallet transactions and balances.
It allows adding transactions to the wallet, updating the wallet balance based on those transactions,
and checking if there are sufficient funds for a trade.

The Wallet struct has two fields: transactions which is a vector of Transaction structs
representing all the transactions for the wallet, and wallets which is a hashmap with keys being
asset symbols and values being Decimal types representing the amount of that asset held in the wallet.

The Wallet struct has several methods:

- new() creates a new Wallet instance with an empty list of
    transactions and an empty hashmap for wallets.
- get_wallets() returns a reference to the wallets hashmap.
- update_wallet() updates the balance of the wallets hashmap based on the given Transaction.
- add() adds a Transaction to the transactions vector and updates the wallet balance based on the
    given Transaction.
- has_funds_for_order() checks if there are sufficient funds for a given asset symbol and required
    amount, and returns the available funds if they are sufficient, otherwise returns None.
*/
#[derive(Debug)]
pub struct Wallet {
    transactions: Vec<Transaction>,
    wallets: HashMap<String, Decimal>,
}

impl Wallet {
    pub fn new() -> Self {
        Wallet {
            transactions: vec![],
            wallets: HashMap::new(),
        }
    }
    pub fn get_wallets(&self) -> &HashMap<String, Decimal> {
        &self.wallets
    }
    pub fn get_transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }
    fn update_wallet(&mut self, tx: &Transaction) {
        let symbol = tx.get_symbol();
        let qty = tx.get_qty();
        self.wallets
            .entry(symbol.clone())
            .and_modify(|e| *e += qty)
            .or_insert(qty.clone());
    }
    pub fn add(&mut self, tx: &Transaction) {
        self.transactions.push(tx.clone());
        self.update_wallet(tx);
    }
    pub fn has_funds_for_order(&self, asset: &str, required_amount: Decimal) -> Option<Decimal> {
        if let Some(funds) = self.wallets.get(asset) {
            if funds >= &required_amount {
                return Some(funds.clone());
            }
        }
        None
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_wallet_adds_to_account() {
        let mut w = Wallet::new();
        w.add(&Transaction::new(
            Utc::now().timestamp(),
            String::from("BTC"),
            dec!(23456.8),
            dec!(1.000000000000000000001),
        ));
        w.add(&Transaction::new(
            Utc::now().timestamp(),
            String::from("BTC"),
            dec!(23456.8),
            dec!(1.000000000000000000001),
        ));
        assert_eq!(
            w.get_wallets().get("BTC").unwrap(),
            &dec!(2.000000000000000000002)
        );
        assert_ne!(
            w.get_wallets().get("BTC").unwrap(),
            &dec!(1.000000000000000000001)
        );
    }

    #[test]
    fn test_wallet_subtracts_from_account() {
        let mut w = Wallet::new();
        // start with 10 BTC
        w.add(&Transaction::new(
            Utc::now().timestamp(),
            String::from("BTC"),
            dec!(23456.8),
            dec!(10),
        ));
        assert_eq!(w.get_wallets().get("BTC").unwrap(), &dec!(10));
        // remove 3 BTC so we should have 7
        w.add(&Transaction::new(
            Utc::now().timestamp(),
            String::from("BTC"),
            dec!(23456.8),
            dec!(-3),
        ));
        assert_eq!(w.get_wallets().get("BTC").unwrap(), &dec!(7));
        // remove another 3 BTC so we should have 4
        w.add(&Transaction::new(
            Utc::now().timestamp(),
            String::from("BTC"),
            dec!(23456.8),
            dec!(-3),
        ));
        assert_eq!(w.get_wallets().get("BTC").unwrap(), &dec!(4));
    }

    #[test]
    fn test_multi_wallet_add_and_subtract() {
        let mut w = Wallet::new();
        // start with 10 BTC and 10 ETH
        w.add(&Transaction::new(
            Utc::now().timestamp(),
            String::from("BTC"),
            dec!(23456.8),
            dec!(10),
        ));
        w.add(&Transaction::new(
            Utc::now().timestamp(),
            String::from("ETH"),
            dec!(1340.0),
            dec!(10),
        ));
        assert_eq!(w.get_wallets().get("BTC").unwrap(), &dec!(10));
        assert_eq!(w.get_wallets().get("ETH").unwrap(), &dec!(10));

        // add 30 ETH so a total of 40 ETH

        w.add(&Transaction::new(
            Utc::now().timestamp(),
            String::from("ETH"),
            dec!(1340.0),
            dec!(30),
        ));

        assert_eq!(w.get_wallets().get("ETH").unwrap(), &dec!(40));

        // add 20 BTC so a total of 30 BTC
        w.add(&Transaction::new(
            Utc::now().timestamp(),
            String::from("BTC"),
            dec!(23456.8),
            dec!(20),
        ));
        assert_eq!(w.get_wallets().get("BTC").unwrap(), &dec!(30));

        // remove 10 ETH and 10 BTC leaving 30 ETH and 20 BTC

        w.add(&Transaction::new(
            Utc::now().timestamp(),
            String::from("ETH"),
            dec!(1340.0),
            dec!(-10),
        ));
        w.add(&Transaction::new(
            Utc::now().timestamp(),
            String::from("BTC"),
            dec!(23456.8),
            dec!(-10),
        ));

        assert_eq!(w.get_wallets().get("ETH").unwrap(), &dec!(30));
        assert_eq!(w.get_wallets().get("BTC").unwrap(), &dec!(20));
    }

    #[test]
    fn test_has_funds_for_order() {
        let mut w = Wallet::new();
        w.add(&Transaction::new(
            0i64,
            String::from("BTC"),
            dec!(0),
            dec!(10),
        ));
        assert_eq!(w.has_funds_for_order("BTC", dec!(9)), Some(dec!(10)));
        let mut w = Wallet::new();
        w.add(&Transaction::new(
            0i64,
            String::from("XRP"),
            dec!(0),
            dec!(10_000),
        ));
        w.add(&Transaction::new(
            0i64,
            String::from("ETH"),
            dec!(0),
            dec!(10),
        ));
        assert_eq!(w.has_funds_for_order("XRP", dec!(12_000)), None);
        assert_eq!(w.has_funds_for_order("ETH", dec!(12_000)), None);
        assert_eq!(
            w.has_funds_for_order("XRP", dec!(9_000)),
            Some(dec!(10_000))
        );
        assert_eq!(w.has_funds_for_order("ETH", dec!(9)), Some(dec!(10)));
    }
}
