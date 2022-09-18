use serde::Serialize;
use std::collections::HashMap;

use crate::error::{Error, Result};

#[derive(Debug, Serialize, Default, Clone)]
pub(crate) struct Account {
    client: u16,
    available: f64,
    held: f64,
    total: f64,
    locked: bool,
}

impl Account {
    pub(crate) fn new(id: u16) -> Self {
        let mut account = Self::default();
        account.client = id;
        account
    }

    pub(crate) fn deposit(&mut self, amount: f64) -> Result<()> {
        self.available += amount;
        Ok(())
    }

    pub(crate) fn withdraw(&mut self, amount: f64) -> Result<()> {
        if self.available - amount < 0 as f64 {
            Err(Error::InsufficientFunds)
        } else {
            self.available -= amount;
            Ok(())
        }
    }

    pub(crate) fn dispute(&mut self, _amount: f64) -> Result<()> {
        todo!()
    }

    pub(crate) fn resolve(&mut self, _amount: f64) -> Result<()> {
        todo!()
    }

    pub(crate) fn chargeback(&mut self, _amount: f64) -> Result<()> {
        todo!()
    }

    pub(crate) fn available(&self) -> f64 {
        self.available
    }
}

#[derive(Debug)]
pub(crate) struct Accounts {
    inner: HashMap<u16, Account>,
}

impl Accounts {
    pub(crate) fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub(crate) fn get(&mut self, id: u16) -> Result<Account> {
        Ok(self.inner.entry(id).or_insert(Account::new(id)).clone())
    }

    pub(crate) fn update(&mut self, account: Account) -> Result<()> {
        self.inner.insert(account.client, account);
        Ok(())
    }
}
