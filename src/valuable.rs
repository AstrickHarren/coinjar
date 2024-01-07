use std::{
    fmt::{Display, Write},
    iter::Sum,
    ops::{Add, AddAssign, Neg},
    sync::Arc,
};

use indenter::indented;
use itertools::Itertools;

#[derive(Debug, Clone, Eq)]
pub(crate) struct Currency {
    name: Option<Arc<String>>,
    symbol: Option<Arc<String>>,
    code: Arc<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Money {
    amount: i32,
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
        let symbol = self.currency.symbol.as_ref().map(|s| s.as_str());
        let positve = self.amount.abs();
        let sign = if self.amount < 0 { "-" } else { "" };

        let major = positve / 100;
        let minor = positve % 100;

        match symbol {
            Some(symbol) => write!(f, "{}{}{}.{:02}", sign, symbol, major, minor),
            None => write!(
                f,
                "{}{}.{:02} {}",
                sign,
                major,
                minor,
                self.currency.code.as_str()
            ),
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
            code: Arc::new(code.to_string().into()),
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
        Self { amount, currency }
    }

    pub(crate) fn from_major(amount: i32, currency: Currency) -> Self {
        Self {
            amount: amount * 100,
            currency,
        }
    }

    pub(crate) fn from_str(mut money: &str, currency_store: &CurrencyStore) -> Option<Self> {
        let is_negative = money.chars().next().unwrap() == '-';
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

        let (major, minor) = amount.split_once('.').unwrap_or((amount, "00"));
        let major = major.parse::<i32>().ok()?;
        let minor = minor.parse::<i32>().ok()?;
        let amount = major * 100 + minor;
        let amount = if is_negative { -amount } else { amount };
        Some(Self { amount, currency })
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
        self.moneys.retain(|m| m.amount != 0);
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.moneys.is_empty()
    }

    pub(crate) fn into_moneys(self) -> impl Iterator<Item = Money> {
        self.moneys.into_iter()
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
    #[rustfmt::skip]
    fn test_round_half_even() {
        // let amuonts = vec![
        //     1.005, 1.015, 1.025, 1.035, 1.045, 1.055, 1.065, 1.075, 1.085, 1.095,
        //     -1.005, -1.015, -1.025, -1.035, -1.045, -1.055, -1.065, -1.075, -1.085, -1.095,
        // ];

        // let rounded = amuonts
        //     .iter()
        //     .map(|a| Money {
        //         amount: *a,
        //         currency: Currency::usd(),
        //     })
        //     .map(|m| m.round_half_even())
        //     .map(|m| m.amount)
        //     .collect::<Vec<_>>();

        // assert_eq!(
        //     rounded,
        //     vec![
        //         1.00, 1.02, 1.02, 1.04, 1.04, 1.06, 1.06, 1.08, 1.08, 1.10,
        //         -1.00, -1.02, -1.02, -1.04, -1.04, -1.06, -1.06, -1.08, -1.08, -1.10,
        //     ]
        // )
    }
}
