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
