use std::{cell::RefCell, collections::HashMap};

use chrono::NaiveDate;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use serde::Deserialize;

use super::{Currency, Money, Valuable};

#[derive(Debug, Deserialize)]
struct RateResponse {
    date: NaiveDate,
    #[serde(flatten)]
    rates: HashMap<String, HashMap<String, f64>>,
}

#[derive(Debug, Deserialize)]
struct Rate {
    date: NaiveDate,
    #[serde(flatten)]
    rate: HashMap<String, f32>,
}

impl RateResponse {
    fn get_for_cur(cur: &str) -> Self {
        let ver = 1;
        let api = "https://cdn.jsdelivr.net/gh/fawazahmed0/currency-api";
        let url = format!("{}@{}/latest/currencies/{}.json", api, ver, cur);
        reqwest::blocking::get(&url).unwrap().json().unwrap()
    }
}

impl Rate {
    fn get_for_pair(from: &str, to: &str) -> Self {
        let ver = 1;
        let api = "https://cdn.jsdelivr.net/gh/fawazahmed0/currency-api";
        let url = format!("{}@{}/latest/currencies/{}/{}.json", api, ver, from, to);
        reqwest::blocking::get(&url).unwrap().json().unwrap()
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct Exchange {
    from: String,
    to: String,
    date: NaiveDate,
}

#[derive(Debug, Default)]
struct ExchangeBook {
    rates: RefCell<HashMap<Exchange, f32>>,
}

impl ExchangeBook {
    fn update_cur_for_pair(&self, from: &Currency, to: &Currency) {
        let from = from.code().to_lowercase();
        let to = to.code().to_lowercase();
        let res = Rate::get_for_pair(&from, &to);

        self.rates.borrow_mut().insert(
            Exchange {
                from: from.to_string(),
                to: to.to_string(),
                date: res.date,
            },
            res.rate[&to],
        );
    }

    fn get(&self, from: &Currency, to: &Currency, date: NaiveDate) -> Option<f32> {
        let exchange = Exchange {
            from: from.code().to_lowercase(),
            to: to.code().to_lowercase(),
            date,
        };
        self.update_cur_for_pair(from, to);
        self.rates.borrow().get(&exchange).copied()
    }
}

impl Money {
    fn convert_to(&self, to: &Currency, date: NaiveDate, book: &ExchangeBook) -> Option<Self> {
        let from = &self.currency;
        let rate = book.get(from, to, date)?;
        let amount = self.amount * Decimal::from_f32(rate)?;

        Some(Self {
            amount,
            currency: to.clone(),
        })
    }
}

impl Valuable {
    fn convert_to(&self, to: &Currency, date: NaiveDate, book: &ExchangeBook) -> Option<Self> {
        self.moneys()
            .map(|money| money.convert_to(to, date, book))
            .try_fold(Valuable::default(), |acc, money| Some(acc + money?))
    }
}

#[cfg(test)]
mod test {
    use chrono::Local;

    use crate::valuable::test::example_currency_store;

    use super::*;

    #[test]
    fn test_get_for_cur() {
        RateResponse::get_for_cur("eur");
        Rate::get_for_pair("eur", "usd");
    }

    #[test]
    fn update_cur() {
        let book = ExchangeBook::default();
        let usd = &Currency::usd();
        let eur = &Currency::eur();
        let today = Local::now().naive_local().date();
        let yesterday = today.pred_opt().unwrap();

        book.update_cur_for_pair(eur, usd);
        dbg!(book.get(eur, usd, yesterday));
    }

    #[test]
    fn conversion() {
        let book = ExchangeBook::default();
        let currency_store = example_currency_store();
        let money = Money::from_str("200 eur", &currency_store).unwrap();
        let target = Currency::usd();

        let today = Local::now().naive_local().date();
        let yesterday = today.pred_opt().unwrap();

        let today_target = money.convert_to(&target, today, &book);
        let yesterday_target = money.convert_to(&target, yesterday, &book);

        println!(
            "{}, {}",
            today_target
                .map(|s| s.to_string())
                .unwrap_or("None".to_string()),
            yesterday_target
                .map(|s| s.to_string())
                .unwrap_or("None".to_string()),
        );
    }

    #[test]
    fn convert_valuable() {
        let book = ExchangeBook::default();
        let currency_store = example_currency_store();
        let usd_100 = Money::from_str("100 eur", &currency_store).unwrap();
        let eur_100 = Money::from_str("100 eur", &currency_store).unwrap();
        let valuable = Valuable::default() + usd_100 + eur_100;

        let today = Local::now().naive_local().date();
        let yesterday = today.pred_opt().unwrap();

        let today_target = valuable.convert_to(&Currency::usd(), today, &book);
        let yesterday_target = valuable.convert_to(&Currency::usd(), yesterday, &book);

        println!(
            "{}, {}",
            today_target
                .map(|s| s.to_string())
                .unwrap_or("None".to_string()),
            yesterday_target
                .map(|s| s.to_string())
                .unwrap_or("None".to_string()),
        );
    }
}
