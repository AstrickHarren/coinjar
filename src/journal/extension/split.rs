use itertools::Itertools;

use crate::{
    accn::{Accn, AccnStore, ContactId},
    journal::{Booking, Posting},
    valuable::{Money, Valuable},
};

use super::BuildBook;

pub(crate) struct Split<B: BuildBook> {
    inner: B,

    // if payer is None, then I am the payer
    payer: Option<ContactId>,
    debtor: Vec<ContactId>,
    expenses: Vec<Posting>,
}

impl<B: BuildBook> BuildBook for Split<B> {
    fn from_booking(booking: Booking) -> Self {
        Self {
            inner: B::from_booking(booking),
            payer: None,
            debtor: Vec::new(),
            expenses: Vec::new(),
        }
    }

    fn with_posting(&mut self, accn: Accn, money: Option<Money>) -> &mut Self {
        if self.is_effective() && accn.is_expense() {
            self.expenses.push(Posting {
                accn: accn.id(),
                money: money.unwrap(),
            });
        } else {
            self.inner.with_posting(accn, money);
        }

        self
    }

    fn with_tag<'a>(
        &mut self,
        accns: &mut AccnStore,
        tag_name: &str,
        args: impl Iterator<Item = &'a str>,
    ) -> &mut Self {
        if tag_name.to_lowercase() == "split" {
            for arg in args {
                match arg.split_once(" ").map(|(a, b)| (a.trim(), b.trim())) {
                    Some(("by", name)) => {
                        let name = name.strip_prefix("@").unwrap();
                        let mut contact = accns.add_contact(name);
                        contact.payable_entry().or_open();
                        match &self.payer {
                            Some(id) => {
                                panic!("payer already set to {}", accns.contact(*id).name())
                            }
                            None => {
                                self.payer = Some(contact.id());
                            }
                        }
                    }
                    None => {
                        let arg = arg.strip_prefix("@").unwrap();
                        let mut contact = accns.add_contact(arg);
                        contact.receivable_entry().or_open();
                        self.debtor.push(contact.id());
                    }
                    _ => unreachable!(),
                }
            }
        };

        self
    }

    fn into_booking_with(mut self, accns: &mut AccnStore) -> Booking {
        let inbalances = self.split_inbalance();
        let moneys = inbalances.moneys().cloned();
        match self.payer {
            // I am the payer, every debtor is a receivable
            None => {
                self.debtor
                    .iter()
                    .cartesian_product(moneys)
                    .for_each(|(id, money)| {
                        self.inner
                            .with_posting(accns.contact(*id).receivable().unwrap(), Some(money));
                    });
            }
            // I am not the payer, only payer is a payable
            Some(payer) => {
                self.inner
                    .with_moneys(accns.contact(payer).payable().unwrap(), moneys.map(|m| -m));
            }
        }

        for p in self.expenses {
            self.inner
                .with_posting(accns.accn(p.accn), Some(p.money.clone()));
        }

        self.inner.into_booking_with(accns)
    }
}

impl<B: BuildBook> Split<B> {
    fn split_inbalance(&mut self) -> Valuable {
        let n_shares = self.n_shares();
        self.expenses
            .iter_mut()
            .map(|p| {
                p.money /= n_shares as i32;
                p.money.clone()
            })
            .sum()
    }

    fn n_shares(&self) -> usize {
        self.payer.into_iter().count() + self.debtor.len() + 1
    }

    fn is_effective(&self) -> bool {
        self.debtor.len() > 0 || self.payer.is_some()
    }
}
