use std::fmt::Display;

use anyhow::bail;
use inquire::Select;

use crate::{
    accn::{AccnEntry, AccnEntryMut},
    util::{Formatted, NotEmpty},
};

use super::*;

pub(crate) fn find_or_create_accn<'a>(
    journal: &'a mut Journal,
    matcher: &'a str,
) -> Result<AccnEntry<'a>> {
    let accn = journal
        .accns()
        .by_name_fuzzy(matcher)
        .map(|accn| accn.id())
        .collect_vec();

    let ret = match accn.len() {
        0 => fuzzy_create_accn(journal, matcher)?.into_ref(),
        1 => accn[0].into_accn(journal.accns()),
        _ => choose(
            accn.into_iter().map(|id| id.into_accn(journal.accns())),
            &format!(
                "{}: {} not unique, choose from candidates",
                "info".green().bold(),
                matcher.blue()
            ),
        )?,
    };

    Ok(ret)
}

fn choose<T: Display>(accns: impl Iterator<Item = T>, prompt: &str) -> Result<T> {
    let items = accns.collect::<Vec<_>>();
    let ret = Select::new(prompt, items).prompt()?;
    Ok(ret)
}

/// Create a new account with the given matcher with the following rules:
/// Suppose the matcher is food:groceries, then:
/// 1. If food:groceries exists, return it
/// 2. If food exists, create food:groceries and return it
/// 3. If food does not exist, return err
fn fuzzy_create_accn<'a>(journal: &'a mut Journal, matcher: &'a str) -> Result<AccnEntryMut<'a>> {
    let original_matcher = matcher;
    let mut matcher = matcher.split(':').collect_vec();
    let mut unmatched = Vec::new();

    // find a match
    #[allow(clippy::never_loop)]
    while let Some(part) = matcher.pop() {
        let _: Option<_> = try {
            unmatched.push(part);
            let formatter = |accn: &AccnEntry| {
                accn.abs_name().to_string() + ":" + &unmatched.iter().rev().join(":")
            };
            let candidates = journal
                .accns()
                .by_name_fuzzy(&matcher)
                .not_empty()?
                .map(|c| Formatted::new(c, &formatter))
                .collect_vec();

            // match found
            let candidate = Select::new(
                &format!("{} not found, create one from candidates", original_matcher),
                candidates,
            )
            .prompt();

            return try {
                let candidate = candidate?;
                let id = candidate.id();
                let mut accn = id.into_accn_mut(journal.accns_mut());

                for part in unmatched.into_iter().rev() {
                    accn = accn.or_open_child(part);
                }

                accn
            };
        };
    }

    bail!("{} not found", original_matcher);
}
