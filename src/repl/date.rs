use std::str::FromStr;

use anyhow::anyhow;
use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub(super) enum DateArg {
    Date(NaiveDate),
    Rel(i32),
}

impl FromStr for DateArg {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rel = s
            .parse::<i32>()
            .ok()
            .map(|n| DateArg::Rel(n))
            .or_else(|| {
                NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .or_else(|_| NaiveDate::parse_from_str(s, "%Y/%m/%d"))
                    .map(|d| DateArg::Date(d))
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
            DateArg::Rel(n) => *date = *date + chrono::Duration::days(*n as i64),
        }
    }
}
