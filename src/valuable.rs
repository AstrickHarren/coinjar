use std::collections::HashMap;

use rust_decimal::Decimal;
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
        store
    }

    fn insert(&mut self, code: String, symbol: String, symbol_first: bool) {
        if let Some(_) = self.get_by_code(&code) {
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
        self.codes.get(code).copied()
    }

    fn get_by_symbol(&self, symbol: &str) -> Option<Currency> {
        self.symbols.get(symbol).copied()
    }
}

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

    pub(crate) fn into_money(self, store: &CurrencyStore) -> Option<Money> {
        let amount = self.amount?;
        let amount = match self.neg {
            true => -amount,
            false => amount,
        };
        let currency = match self.code {
            Some(code) => store.get_by_code(code)?,
            None => store.get_by_symbol(self.symbol?)?,
        };
        Some(Money { amount, currency })
    }
}
