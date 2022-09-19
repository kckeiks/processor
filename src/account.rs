use serde::Serialize;
use std::collections::HashMap;

use crate::error::{Error, Result};

/// Account is responsible for updating values on account.
#[derive(Debug, Serialize, Default, Clone)]
pub(crate) struct Account {
    client: u16,
    available: f64,
    held: f64,
    total: f64,
    locked: bool,
}

impl Account {
    fn new(id: u16) -> Self {
        let mut account = Self::default();
        account.client = id;
        account
    }

    fn deposit(&mut self, amount: f64) -> Result<()> {
        self.available += amount;
        self.total += amount;
        Ok(())
    }

    fn hold(&mut self, amount: f64) -> Result<()> {
        self.held += amount;
        self.total += amount;
        Ok(())
    }

    fn withdraw(&mut self, amount: f64) -> Result<()> {
        if self.available - amount < 0 as f64 {
            Err(Error::InsufficientFunds)
        } else {
            self.available -= amount;
            self.total -= amount;
            Ok(())
        }
    }

    fn withdraw_held(&mut self, amount: f64) -> Result<()> {
        if self.held - amount < 0 as f64 {
            Err(Error::InsufficientFunds)
        } else {
            self.held -= amount;
            self.total -= amount;
            Ok(())
        }
    }

    fn dispute(&mut self, amount: f64) -> Result<()> {
        self.withdraw(amount)?;
        self.hold(amount)?;
        Ok(())
    }

    fn resolve(&mut self, amount: f64) -> Result<()> {
        self.withdraw_held(amount)?;
        self.deposit(amount)?;
        Ok(())
    }

    fn chargeback(&mut self, amount: f64) -> Result<()> {
        self.withdraw_held(amount)?;
        self.locked = true;
        Ok(())
    }

    pub(crate) fn frozen(&self) -> bool {
        self.locked
    }

    #[cfg(test)]
    pub(crate) fn available(&self) -> f64 {
        self.available
    }

    #[cfg(test)]
    pub(crate) fn total(&self) -> f64 {
        self.total
    }
}

/// Accounts provides functionality to make
/// updates to individual accounts and transactions.
#[derive(Debug)]
pub(crate) struct Accounts {
    inner: HashMap<u16, Account>,
    transactions: HashMap<u32, Transaction>,
}

impl Accounts {
    pub(crate) fn new() -> Self {
        Self {
            inner: HashMap::new(),
            transactions: HashMap::new(),
        }
    }

    pub(crate) fn account(&mut self, id: u16) -> Result<Account> {
        Ok(self.inner.entry(id).or_insert(Account::new(id)).clone())
    }

    fn transaction(&self, id: u32) -> Option<Transaction> {
        self.transactions.get(&id).cloned()
    }

    fn update_account(&mut self, account: Account) -> Result<()> {
        self.inner.insert(account.client, account);
        Ok(())
    }

    pub(crate) fn deposit(&mut self, client: u16, amount: f64, tx: u32) -> Result<()> {
        let mut account = self.account(client)?;
        if account.frozen() {
            return Ok(());
        }
        account.deposit(amount)?;
        self.update_account(account)?;
        // Record transaction.
        self.update_transaction(Transaction::new(tx, amount))?;
        Ok(())
    }

    pub(crate) fn withdraw(&mut self, client: u16, amount: f64, tx: u32) -> Result<()> {
        let mut account = self.account(client)?;
        if account.frozen() {
            return Ok(());
        }
        account.withdraw(amount)?;
        self.update_account(account)?;
        // Record transaction.
        self.update_transaction(Transaction::new(tx, amount))?;
        Ok(())
    }

    pub(crate) fn dispute(&mut self, client: u16, tx: u32) -> Result<()> {
        if let Some(mut trans) = self.transaction(tx) {
            let mut account = self.account(client)?;
            if account.frozen() {
                return Ok(());
            }

            if let Status::Open = trans.status {
                if let Err(Error::InsufficientFunds) = account.dispute(trans.amount) {
                    return Ok(());
                }
                trans.status = Status::Pending;
                let new_trans = trans.clone();
                self.update_transaction(new_trans)?;
            }

            self.update_account(account)?;
        }
        Ok(())
    }

    pub(crate) fn resolve(&mut self, client: u16, tx: u32) -> Result<()> {
        if let Some(mut trans) = self.transaction(tx) {
            let mut account = self.account(client)?;
            if account.frozen() {
                return Ok(());
            }

            if let Status::Pending = trans.status {
                if let Err(Error::InsufficientFunds) = account.resolve(trans.amount) {
                    return Ok(());
                }
                trans.status = Status::Resolved;
                let new_trans = trans.clone();
                self.update_transaction(new_trans)?;
            }

            self.update_account(account)?;
        }
        Ok(())
    }

    pub(crate) fn chargeback(&mut self, client: u16, tx: u32) -> Result<()> {
        if let Some(mut trans) = self.transaction(tx) {
            let mut account = self.account(client)?;
            if account.frozen() {
                return Ok(());
            }

            if let Status::Pending = trans.status {
                if let Err(Error::InsufficientFunds) = account.chargeback(trans.amount) {
                    return Ok(());
                }
                trans.status = Status::Chargeback;
                let new_trans = trans.clone();
                self.update_transaction(new_trans)?;
            }

            self.update_account(account)?;
        }
        Ok(())
    }

    fn update_transaction(&mut self, tx: Transaction) -> Result<()> {
        self.transactions.insert(tx.id, tx);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Status {
    Open,
    Pending,
    Resolved,
    Chargeback,
}

#[derive(Debug, Clone)]
struct Transaction {
    id: u32,
    amount: f64,
    status: Status,
}

impl Transaction {
    fn new(id: u32, amount: f64) -> Self {
        Self {
            id,
            amount,
            status: Status::Open,
        }
    }
}
