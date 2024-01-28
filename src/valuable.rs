use std::{
    collections::HashMap,
    fmt::Display,
    iter::Sum,
    ops::{Add, AddAssign, Neg},
};

use anyhow::{anyhow, Result};
use itertools::Itertools;
use rust_decimal::{
    prelude::{Signed, ToPrimitive, Zero},
    Decimal, RoundingStrategy,
};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Currency {
    id: Uuid,
}

impl Currency {
    fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }
}

#[derive(Debug)]
struct CurrencyData {
    code: String,
    symbol: Option<String>,
    symbol_first: bool,
}

#[derive(Debug, Default)]
pub(crate) struct CurrencyStore {
    codes: HashMap<String, Currency>,
    symbols: HashMap<String, Currency>,
    currencies: HashMap<Currency, CurrencyData>,
}

impl CurrencyStore {
    pub(crate) fn new() -> Self {
        let mut store = Self::default();
        store.insert("USD".to_string(), "$".to_string(), true);
        store.insert("GBP".to_string(), "£".to_string(), false);
        store.insert("EUR".to_string(), "€".to_string(), true);
        store.insert("RUB".to_string(), "₽".to_string(), false);
        store.insert("CNY".to_string(), "¥".to_string(), true);
        store.insert("BTC".to_string(), "₿".to_string(), true);
        store
    }

    fn insert(&mut self, code: String, symbol: String, symbol_first: bool) {
        if self.get_by_code(&code).is_some() {
            return;
        }

        let currency = Currency::new();
        let data = CurrencyData {
            code: code.clone(),
            symbol: Some(symbol.clone()),
            symbol_first,
        };

        self.codes.insert(code, currency);
        self.symbols.insert(symbol.clone(), currency);
        self.currencies.insert(currency, data);
    }

    fn get_by_code(&self, code: &str) -> Option<Currency> {
        // WARNING: Assuming all codes are uppercase.
        self.codes.get(&code.to_uppercase()).copied()
    }

    fn get_by_symbol(&self, symbol: &str) -> Option<Currency> {
        self.symbols.get(symbol).copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Money {
    amount: Decimal,
    currency: Currency,
}

impl Money {
    pub(crate) fn fmt(&self, store: &CurrencyStore) -> String {
        let data = store.currencies.get(&self.currency).unwrap();
        let s = data.symbol.as_ref().unwrap();
        let symbol_first = data.symbol_first;

        let sign = match self.amount.is_sign_positive() {
            true => "",
            false => "-",
        };

        match symbol_first {
            true => format!("{}{}{}", sign, s, self.amount.abs()),
            false => format!("{}{}{}", sign, self.amount.abs(), s),
        }
    }

    fn zero(currency: Currency) -> Self {
        Self {
            amount: Decimal::zero(),
            currency,
        }
    }

    fn new(amount: Decimal, currency: Currency) -> Self {
        Self { amount, currency }
    }

    pub(super) fn eq_currency(&self, other: &Self) -> bool {
        self.currency == other.currency
    }

    pub(crate) fn into_money(self, store: &CurrencyStore) -> MoneyEntry {
        MoneyEntry { money: self, store }
    }

    /// Split money into n parts, each with dp decimal places, guaranteeing that
    /// the sum of the parts is equal to the original amount, and that the
    /// difference between the largest and smallest part is less than or equal
    /// to 1e-dp.
    pub(crate) fn split(self, n: usize, dp: u32) -> impl Iterator<Item = Self> {
        let amount: Decimal = self.amount / Decimal::from(n);
        let amount = amount.round_dp_with_strategy(dp, RoundingStrategy::MidpointNearestEven);
        let remainder: Decimal = self.amount - amount * Decimal::from(n);
        let signum = remainder.signum();

        let complement = Decimal::from_scientific(&format!("1e-{}", dp)).unwrap() * signum;
        let n_complements = match complement.is_zero() {
            true => 0,
            false => (remainder / complement).abs().to_usize().unwrap(),
        };

        std::iter::repeat(amount)
            .take(n)
            .enumerate()
            .map(move |(i, amount)| match i < n_complements {
                true => amount + complement,
                false => amount,
            })
            .map(move |amount| Self::new(amount, self.currency))
    }
}

impl Neg for Money {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            amount: -self.amount,
            currency: self.currency,
        }
    }
}

impl AddAssign for Money {
    fn add_assign(&mut self, rhs: Self) {
        debug_assert!(self.eq_currency(&rhs));
        self.amount += rhs.amount;
    }
}

pub(crate) struct MoneyEntry<'a> {
    money: Money,
    store: &'a CurrencyStore,
}

impl MoneyEntry<'_> {
    pub(crate) fn money(&self) -> Money {
        self.money
    }
}

impl From<MoneyEntry<'_>> for Money {
    fn from(entry: MoneyEntry) -> Self {
        entry.money
    }
}

impl Display for MoneyEntry<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.money.fmt(self.store))
    }
}

#[derive(Debug, Default)]
pub(crate) struct MoneyBuilder<'a> {
    symbol: Option<&'a str>,
    amount: Option<Decimal>,
    code: Option<&'a str>,
    neg: bool,
}

impl<'a> MoneyBuilder<'a> {
    pub(crate) fn neg(&mut self) -> &mut Self {
        self.neg = true;
        self
    }

    pub(crate) fn with_symbol(&mut self, symbol: &'a str) -> &mut Self {
        self.symbol = Some(symbol);
        self
    }

    pub(crate) fn with_amount(&mut self, amount: Decimal) -> &mut Self {
        self.amount = Some(amount);
        self
    }

    pub(crate) fn with_code(&mut self, code: &'a str) -> &mut Self {
        self.code = Some(code);
        self
    }

    pub(crate) fn into_money(self, store: &CurrencyStore) -> Result<Money> {
        let amount = self.amount.ok_or_else(|| anyhow!("amount missing"))?;
        let amount = match self.neg {
            true => -amount,
            false => amount,
        };
        let currency = match self.code {
            Some(code) => store
                .get_by_code(code)
                .ok_or_else(|| anyhow!("code {} not found", code))?,
            None => {
                let symbol = self
                    .symbol
                    .ok_or_else(|| anyhow!("currency code or symbol missing"))?;
                store
                    .get_by_symbol(symbol)
                    .ok_or_else(|| anyhow!("symbol {} not found", symbol))?
            }
        };
        Ok(Money { amount, currency })
    }
}

#[derive(Debug, Default)]
pub(crate) struct Valuable {
    moneys: HashMap<Currency, Money>,
}

impl IntoIterator for Valuable {
    type Item = Money;
    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.moneys.into_values()
    }
}

impl Zero for Valuable {
    fn zero() -> Self {
        Self::default()
    }

    fn is_zero(&self) -> bool {
        self.moneys.is_empty()
    }
}

impl AddAssign<Money> for Valuable {
    fn add_assign(&mut self, rhs: Money) {
        let currency = rhs.currency;
        let money = self
            .moneys
            .entry(currency)
            .and_modify(|money| money.amount += rhs.amount)
            .or_insert_with(|| rhs);
        if money.amount.is_zero() {
            self.moneys.remove(&currency);
        }
    }
}

impl Add<Money> for Valuable {
    type Output = Self;
    fn add(mut self, rhs: Money) -> Self::Output {
        self += rhs;
        self
    }
}

impl Add<Valuable> for Valuable {
    type Output = Self;
    fn add(mut self, rhs: Valuable) -> Self::Output {
        for (_, money) in rhs.moneys {
            self += money;
        }
        self
    }
}

impl Sum<Money> for Valuable {
    fn sum<I: Iterator<Item = Money>>(iter: I) -> Self {
        let mut valuable = Self::default();
        for money in iter {
            valuable += money;
        }
        valuable
    }
}

#[derive(Default)]
pub(crate) struct ValuableEntry<'a> {
    valuable: HashMap<Currency, MoneyEntry<'a>>,
}

impl<'a> AddAssign<MoneyEntry<'a>> for ValuableEntry<'a> {
    fn add_assign(&mut self, rhs: MoneyEntry<'a>) {
        let currency = rhs.money.currency;
        let money = self
            .valuable
            .entry(currency)
            .and_modify(|money| money.money.amount += rhs.money.amount)
            .or_insert_with(|| rhs);
        if money.money.amount.is_zero() {
            self.valuable.remove(&currency);
        }
    }
}

impl<'a> Add<MoneyEntry<'a>> for ValuableEntry<'a> {
    type Output = Self;
    fn add(mut self, rhs: MoneyEntry<'a>) -> Self::Output {
        self += rhs;
        self
    }
}

impl<'a> Sum<MoneyEntry<'a>> for ValuableEntry<'a> {
    fn sum<I: Iterator<Item = MoneyEntry<'a>>>(iter: I) -> Self {
        let mut valuable = Self::default();
        for money in iter {
            valuable += money;
        }
        valuable
    }
}

impl<'a> Neg for ValuableEntry<'a> {
    type Output = Self;
    fn neg(mut self) -> Self::Output {
        for money in self.valuable.values_mut() {
            money.money = -money.money;
        }
        self
    }
}

impl Display for ValuableEntry<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.valuable.is_empty() {
            true => write!(f, "{}", 0),
            false => self.valuable.values().format(", ").fmt(f),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_split() {
        let de = dec!(100.00);
        let dp = 11;
        let n = 11;
        let precision = Decimal::from_scientific(&format!("1e-{}", dp)).unwrap();

        let money = Money::new(de, Currency::new());
        let moneys: Vec<_> = money.split(n, dp).map(|money| money.amount).collect();

        dbg!(&moneys);
        let sum = moneys.iter().sum::<Decimal>();
        let max = moneys.iter().max().unwrap();
        let min = moneys.iter().min().unwrap();

        assert_eq!(sum, de);
        assert!(max - min <= precision);
    }
}
