use std::str::FromStr;

use anyhow::{anyhow, Result};
use chrono::{Datelike, NaiveDate};

#[derive(Debug, Clone)]
pub(super) enum DateArg {
    Date(NaiveDate),
    Rel(i32),
}

impl DateArg {
    fn parse_no_year(s: &str, fmt: &str) -> Result<Self> {
        let today = chrono::Local::now().date_naive();
        let fmt = format!("%Y-{}", fmt);

        try {
            let date = format!("{}-{}", today.year(), s);
            let mut date = NaiveDate::parse_from_str(&date, &fmt)?;
            if date > today {
                let s = format!("{}-{}", today.year() - 1, s);
                date = NaiveDate::parse_from_str(&s, &fmt).unwrap();
            }
            Self::Date(date)
        }
    }
}

impl FromStr for DateArg {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rel = s
            .parse::<i32>()
            .ok()
            .map(DateArg::Rel)
            .or_else(|| {
                NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .or_else(|_| NaiveDate::parse_from_str(s, "%Y/%m/%d"))
                    .map(DateArg::Date)
                    .or_else(|_| DateArg::parse_no_year(s, "%m-%d"))
                    .or_else(|_| DateArg::parse_no_year(s, "%m/%d"))
                    .ok()
            })
            .or_else(|| {
                let today = chrono::Local::now().date_naive();
                let day = match s {
                    "today" => Some(today),
                    "yesterday" => today.pred_opt(),
                    "tomorrow" => today.succ_opt(),
                    _ => None,
                }?;
                Some(DateArg::Date(day))
            });

        rel.ok_or_else(|| anyhow!("invalid date: {}", s))
    }
}

impl DateArg {
    pub(super) fn apply(&self, date: &mut NaiveDate) {
        match self {
            DateArg::Date(d) => *date = *d,
            DateArg::Rel(n) => *date += chrono::Duration::days(*n as i64),
        }
    }
}
