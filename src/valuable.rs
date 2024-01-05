use std::{fmt::Display, sync::Arc};

#[derive(Debug)]
pub(crate) struct Currency {
    name: Option<&'static str>,
    symbol: Option<&'static str>,
    code: Arc<&'static str>,
}

#[derive(Debug)]
pub(crate) struct Money {
    amount: f32,
    currency: Currency,
}

struct Valuable {
    moneys: Vec<Money>,
}

#[derive(Debug, Default)]
pub(crate) struct CurrencyStore {
    currencies: Vec<Currency>,
}

impl Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbol = self.currency.symbol.unwrap_or("");
        match self.amount < 0.0 {
            true => write!(f, "-{}{}", symbol, -self.amount),
            false => write!(f, " {}{}", symbol, self.amount),
        }
    }
}

impl Currency {
    pub(crate) fn usd() -> Self {
        Self {
            name: Some("US Dollar"),
            symbol: Some("$"),
            code: Arc::new("USD"),
        }
    }

    pub(crate) fn eur() -> Self {
        Self {
            name: Some("Euro"),
            symbol: Some("€"),
            code: Arc::new("EUR"),
        }
    }

    pub(crate) fn cny() -> Self {
        Self {
            name: Some("Chinese Yuan"),
            symbol: Some("¥"),
            code: Arc::new("CNY"),
        }
    }

    pub(crate) fn jpy() -> Self {
        Self {
            name: Some("Japanese Yen"),
            symbol: Some("¥"),
            code: Arc::new("JPY"),
        }
    }
}

impl Money {
    pub(crate) fn from_minor(amount: i32, currency: Currency) -> Self {
        Self {
            amount: amount as f32 / 100.0,
            currency,
        }
    }

    pub(crate) fn from_major(amount: i32, currency: Currency) -> Self {
        Self {
            amount: amount as f32,
            currency,
        }
    }
}

#[cfg(test)]
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
}
