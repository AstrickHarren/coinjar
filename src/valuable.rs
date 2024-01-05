use std::{fmt::Display, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Currency {
    name: Option<Arc<String>>,
    symbol: Option<Arc<String>>,
    code: Arc<String>,
}

#[derive(Debug, PartialEq)]
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
        let symbol = self
            .currency
            .symbol
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("");
        match self.amount < 0.0 {
            true => write!(f, "-{}{:.2}", symbol, -self.amount),
            false => write!(f, "{}{:.2}", symbol, self.amount),
        }
    }
}

impl Currency {
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

    pub(crate) fn from_str(mut money: &str, currency_store: &CurrencyStore) -> Self {
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
                let amount = chars.as_str().parse::<f32>().unwrap();
                let currency = currency_store.currency_by_symbol(&symbol).unwrap();
                (amount, currency.clone())
            }
            Some(last) => {
                // 2. currency code is last (e.g. -100.00 USD)
                let amount = first.parse::<f32>().unwrap();
                let currency = currency_store.currency_by_code(last).unwrap();
                (amount, currency.clone())
            }
        };

        let amount = if is_negative { -amount } else { amount };
        Self { amount, currency }
    }
}

impl CurrencyStore {
    fn currency_by_code(&self, code: &str) -> Option<&Currency> {
        self.currencies.iter().find(|c| c.code.as_ref() == code)
    }

    fn currency_by_symbol(&self, symbol: &str) -> Option<&Currency> {
        self.currencies
            .iter()
            .find(|c| c.symbol.as_ref().map(|s| s.as_str()) == Some(symbol))
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
        let usd = Money::from_str("-$100.00", &currency_store);
        let usd_ = Money::from_str("-100.00 USD", &currency_store);
        println!("{}", usd);
        assert_eq!(usd, usd_);

        let eur = Money::from_str("€1000", &currency_store);
        let eur_ = Money::from_str("1000 EUR", &currency_store);
        println!("{}", eur);
        assert_eq!(eur, eur_);
    }
}
