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

    fn with_tag(
        &mut self,
        accns: &mut AccnStore,
        tag_name: &str,
        args: impl Iterator<Item = impl AsRef<str>>,
    ) -> &mut Self {
        if tag_name.to_lowercase() == "split" {
            for arg in args {
                match arg
                    .as_ref()
                    .split_once(' ')
                    .map(|(a, b)| (a.trim(), b.trim()))
                {
                    Some(("by", name)) => {
                        let name = name.strip_prefix('@').unwrap();
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
                        let arg = arg.as_ref().strip_prefix('@').unwrap();
                        let mut contact = accns.add_contact(arg);
                        contact.receivable_entry().or_open();
                        self.debtor.push(contact.id());
                    }
                    _ => unreachable!(),
                }
            }
        } else {
            self.inner.with_tag(accns, tag_name, args);
        }

        self
    }

    fn into_booking_with(mut self, accns: &mut AccnStore) -> Booking {
        match self.payer {
            // I am the payer, every debtor is a receivable
            None => {
                let splits = self.split().collect_vec();
                self.debtor
                    .iter()
                    .zip_eq(&splits)
                    .flat_map(|(id, val)| val.moneys().map(move |m| (id, m)))
                    .for_each(|(id, money)| {
                        self.inner.with_posting(
                            accns.contact(*id).receivable().unwrap(),
                            Some(money.clone()),
                        );
                    });
            }
            // I am not the payer, only payer is a payable
            Some(payer) => {
                let moneys = self.split_once();
                let moneys = moneys.moneys();
                self.inner.with_moneys(
                    accns.contact(payer).payable().unwrap(),
                    moneys.map(|m| -m.clone()),
                );
            }
        }

        for p in self.expenses {
            self.inner
                .with_posting(accns.accn(p.accn), Some(p.money.clone()));
        }

        self.inner.into_booking_with(accns)
    }

    fn parse_accn<'a>(
        &mut self,
        accns: &'a mut AccnStore,
        names: impl IntoIterator<Item = impl std::borrow::Borrow<str>>,
    ) -> crate::accn::AccnMut<'a> {
        self.inner.parse_accn(accns, names)
    }
}

impl<B: BuildBook> Split<B> {
    fn split_once(&mut self) -> Valuable {
        let n_shares = self.n_shares();
        self.expenses
            .iter_mut()
            .map(|p| {
                p.money = (p.money.clone() / n_shares as i32).round();
                p.money.clone()
            })
            .sum()
    }

    fn split(&mut self) -> impl Iterator<Item = Valuable> + '_ {
        let n_shares = self.n_shares();
        let groups = &self
            .expenses
            .iter_mut()
            .flat_map(move |p| {
                let mut iter = p.money.clone().split_rounded(n_shares);
                p.money = iter.next().unwrap();
                iter.enumerate()
            })
            .group_by(|(i, _)| *i);
        let vals = groups
            .into_iter()
            .map(|(_, g)| g.map(|(_, m)| m).sum::<Valuable>());
        vals.collect_vec().into_iter()
    }

    fn n_shares(&self) -> usize {
        self.payer.into_iter().count() + self.debtor.len() + 1
    }

    fn is_effective(&self) -> bool {
        !self.debtor.is_empty() || self.payer.is_some()
    }
}
