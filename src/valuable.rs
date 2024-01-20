pub(crate) mod conversion;

use std::{
    fmt::{Display, Write},
    iter::Sum,
    ops::{Add, AddAssign, Div, DivAssign, Neg},
    str::FromStr,
    sync::Arc,
};

use indenter::indented;
use itertools::Itertools;
use rust_decimal::{
    prelude::{Signed, ToPrimitive},
    Decimal, RoundingStrategy,
};
use rust_decimal_macros::dec;

#[derive(Debug, Clone, Eq)]
pub(crate) struct Currency {
    name: Option<Arc<String>>,
    symbol: Option<Arc<String>>,
    code: Arc<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Money {
    amount: Decimal,
    currency: Currency,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct Valuable {
    moneys: Vec<Money>,
}

#[derive(Debug, Default)]
pub(crate) struct CurrencyStore {
    currencies: Vec<Currency>,
}

impl Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let positve = self
            .amount
            .abs()
            .round_dp_with_strategy(2, RoundingStrategy::MidpointNearestEven);
        let sign = if self.amount < 0.into() { "-" } else { "" };
        let symbol = self.currency.symbol.as_ref().map(|s| s.as_str());

        match symbol {
            Some(symbol) => write!(f, "{}{}{:.02}", sign, symbol, positve),
            None => write!(f, "{}{:.02} {}", sign, positve, self.currency.code.as_str()),
        }
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

impl Currency {
    pub(crate) fn new(
        name: Option<impl ToString>,
        symbol: Option<impl ToString>,
        code: impl ToString,
    ) -> Self {
        Self {
            name: name.map(|n| n.to_string().into()),
            symbol: symbol.map(|s| s.to_string().into()),
            code: Arc::new(code.to_string()),
        }
    }

    pub(crate) fn usd() -> Self {
        Self {
            name: Some(Arc::new("US Dollar".to_string())),
            symbol: Some(Arc::new("$".to_string())),
            code: Arc::new("USD".to_string()),
        }
    }

    pub(crate) fn eur() -> Self {
        Self {
            name: Some(Arc::new("Euro".to_string())),
            symbol: Some(Arc::new("€".to_string())),
            code: Arc::new("EUR".to_string()),
        }
    }

    pub(crate) fn cny() -> Self {
        Self {
            name: Some(Arc::new("Chinese Yuan".to_string())),
            symbol: Some(Arc::new("¥".to_string())),
            code: Arc::new("CNY".to_string()),
        }
    }

    pub(crate) fn jpy() -> Self {
        Self {
            name: Some(Arc::new("Japanese Yen".to_string())),
            symbol: Some(Arc::new("¥".to_string())),
            code: Arc::new("JPY".to_string()),
        }
    }

    pub(crate) fn code(&self) -> &str {
        self.code.as_str()
    }
}

impl PartialEq for Currency {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name.as_ref().map(|n| n.as_str()).unwrap_or_default();
        let symbol = self.symbol.as_ref().map(|s| s.as_str()).unwrap_or_default();
        let code = self.code.as_str();

        match f.alternate() {
            true => write!(f, "{} ({}, {})", name, symbol, self.code.as_str()),
            false => write!(f, "{} {} -- {}", code, symbol, name),
        }
    }
}

impl Money {
    pub(crate) fn from_minor(amount: i32, currency: Currency) -> Self {
        Self {
            amount: amount.into(),
            currency,
        }
    }

    pub(crate) fn from_major(amount: i32, currency: Currency) -> Self {
        Self {
            amount: Decimal::from(amount),
            currency,
        }
    }

    pub(crate) fn from_str(mut money: &str, currency_store: &CurrencyStore) -> Option<Self> {
        let is_negative = money.starts_with('-');
        if is_negative {
            money = &money[1..];
        }

        let mut parts = money.split_whitespace();
        let first = parts.next().unwrap();
        let last = parts.last();

        let (amount, currency) = match last {
            None => {
                // 1. currency symbol is first (e.g. -$100.00)
                let mut chars = first.chars();
                let symbol = chars.next().unwrap().to_string();
                let amount = chars.as_str();
                let currency = currency_store.currency_by_symbol(&symbol).unwrap();
                (amount, currency.clone())
            }
            Some(last) => {
                // 2. currency code is last (e.g. -100.00 USD)
                let amount = first;
                let currency = currency_store.currency_by_code(&last.to_uppercase())?;
                (amount, currency.clone())
            }
        };

        let mut amount = Decimal::from_str(amount).ok()?;
        amount = if is_negative { -amount } else { amount };
        Some(Self { amount, currency })
    }

    pub(crate) fn round(mut self) -> Money {
        self.amount = self
            .amount
            .round_dp_with_strategy(2, RoundingStrategy::MidpointNearestEven);
        self
    }

    pub(crate) fn split_rounded(self, n: usize) -> impl Iterator<Item = Money> {
        let share = (self.clone() / n as i32).round().amount;
        let remainder = self.amount - share * Decimal::from(n);
        let sign = remainder.signum();
        let n_compensations = (remainder.abs() / dec!(0.01)).round().to_usize().unwrap();
        debug_assert!(n_compensations <= n);

        std::iter::repeat(share)
            .take(n)
            .enumerate()
            .map(move |(i, mut share)| {
                if i < n_compensations {
                    share += sign * dec!(0.01);
                }
                Money {
                    amount: share,
                    currency: self.currency.clone(),
                }
            })
    }
}

impl Add<Money> for Money {
    type Output = Money;

    fn add(self, rhs: Money) -> Self::Output {
        if rhs.currency != self.currency {
            panic!("cannot add money with different currency");
        }

        Self {
            amount: self.amount + rhs.amount,
            currency: self.currency,
        }
    }
}

impl AddAssign<Money> for Money {
    fn add_assign(&mut self, rhs: Money) {
        if rhs.currency != self.currency {
            panic!("cannot add money with different currency");
        }

        self.amount += rhs.amount;
    }
}

impl Div<i32> for Money {
    type Output = Money;

    fn div(mut self, rhs: i32) -> Self::Output {
        self /= rhs;
        self
    }
}

impl DivAssign<i32> for Money {
    fn div_assign(&mut self, rhs: i32) {
        self.amount /= Decimal::from(rhs)
    }
}

impl CurrencyStore {
    pub(crate) fn new() -> Self {
        Self {
            currencies: Vec::new(),
        }
    }

    fn currency_by_code(&self, code: &str) -> Option<&Currency> {
        self.currencies.iter().find(|c| c.code.as_ref() == code)
    }

    fn currency_by_symbol(&self, symbol: &str) -> Option<&Currency> {
        self.currencies
            .iter()
            .find(|c| c.symbol.as_ref().map(|s| s.as_str()) == Some(symbol))
    }

    pub(crate) fn add_currency(
        &mut self,
        desc: Option<impl ToString>,
        symbol: Option<impl ToString>,
        code: impl ToString,
    ) {
        let currency = Currency::new(desc, symbol, code);
        if !self.currencies.contains(&currency) {
            self.currencies.push(currency);
        }
    }
}

impl Display for CurrencyStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "currency")?;
        let ret = self
            .currencies
            .iter()
            .sorted_by_key(|c| c.code.as_str())
            .format("\n");

        write!(indented(f), "{}", ret)
    }
}

impl Valuable {
    pub(crate) fn add_money(&mut self, money: Money) {
        let m = self
            .moneys
            .iter_mut()
            .find(|m| m.currency == money.currency);

        match m {
            Some(m) => *m += money,
            None => self.moneys.push(money),
        }
        self.simplify()
    }

    pub(crate) fn simplify(&mut self) {
        self.moneys.retain(|m| !m.amount.is_zero());
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.moneys.is_empty()
    }

    pub(crate) fn into_moneys(self) -> impl Iterator<Item = Money> {
        self.moneys.into_iter()
    }

    pub(crate) fn moneys(&self) -> impl Iterator<Item = &Money> + Clone {
        self.moneys.iter()
    }
}

impl From<Money> for Valuable {
    fn from(money: Money) -> Self {
        let mut valuable = Self::default();
        valuable.add_money(money);
        valuable
    }
}

impl Sum<Money> for Valuable {
    fn sum<I: Iterator<Item = Money>>(iter: I) -> Self {
        let mut valuable = Self::default();
        for money in iter {
            valuable.add_money(money);
        }
        valuable
    }
}

impl AddAssign for Valuable {
    fn add_assign(&mut self, rhs: Self) {
        for money in rhs.moneys {
            self.add_money(money);
        }
    }
}

impl Add<Money> for Valuable {
    type Output = Valuable;

    fn add(mut self, rhs: Money) -> Self::Output {
        self.add_money(rhs);
        self
    }
}

impl AddAssign<Money> for Valuable {
    fn add_assign(&mut self, rhs: Money) {
        self.add_money(rhs);
    }
}

impl Display for Valuable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.moneys.is_empty() {
            return write!(f, "0.00");
        }

        self.moneys
            .iter()
            .sorted_by_key(|m| m.currency.code.as_str())
            .format("\n")
            .fmt(f)
    }
}

// TODO: cfg test
pub(crate) mod test {

    use super::*;

    pub(crate) fn example_currency_store() -> CurrencyStore {
        let currencies = vec![
            Currency::usd(),
            Currency::eur(),
            Currency::cny(),
            Currency::jpy(),
        ];
        CurrencyStore { currencies }
    }

    #[test]
    fn test_parse_money() {
        let currency_store = example_currency_store();
        let usd = Money::from_str("-$100.00", &currency_store).unwrap();
        let usd_ = Money::from_str("-100.00 USD", &currency_store).unwrap();
        println!("{}", usd);
        assert_eq!(usd, usd_);

        let eur = Money::from_str("€1000", &currency_store).unwrap();
        let eur_ = Money::from_str("1000 EUR", &currency_store).unwrap();
        println!("{}", eur);
        assert_eq!(eur, eur_);
    }

    #[test]
    fn test_single_round() {
        let amount = 30_26;
        let rounded = Money::from_minor(amount, Currency::usd()) / 2;
        assert_eq!(rounded.amount, 15_13.into());
    }

    #[test]
    #[rustfmt::skip]
    fn test_round_half_even() {
        let positve_amounts = 20_01..=20_20;
        let rounded_positve = rounded(positve_amounts.clone());
        let rounded_negtive = rounded(positve_amounts.clone().map(|a| -a));
        let half_positive = positve_amounts.map(|a| (a as f32)/2.0);
        let expected_positive = half_positive.map(
            |a| {
                let integer_part = a.floor() as i32;
                let decimal_part = a - integer_part as f32;

                if decimal_part > 0.5 || (decimal_part == 0.5 && integer_part % 2 == 1) {
                    integer_part + 1
                } else {
                    integer_part
                }
            }
        );
        let expected_negative = expected_positive.clone().map(|a| -a);
        assert_eq!(rounded_positve, expected_positive.into_iter().map(Into::into).collect::<Vec<_>>());
        assert_eq!(rounded_negtive, expected_negative.into_iter().map(Into::into).collect::<Vec<_>>());
    }

    fn rounded(amounts: impl Iterator<Item = i32>) -> Vec<Decimal> {
        amounts
            .map(|a| Money {
                amount: a.into(),
                currency: Currency::usd(),
            })
            .map(|m| (m / 100 / 2).round().amount * Decimal::from(100))
            .collect::<Vec<_>>()
    }

    #[test]
    fn test_split() {
        let money = Money::from_minor(100, Currency::usd());
        let split = money
            .clone()
            .split_rounded(7)
            .map(|m| m.amount)
            .collect::<Vec<_>>();
        dbg!(&split);
        assert_eq!(money.amount, split.iter().sum::<Decimal>());
    }
}
