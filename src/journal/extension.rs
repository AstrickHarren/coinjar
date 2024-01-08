pub(crate) mod fuzzy_accn;
pub(crate) mod relative_date;
pub(crate) mod split;

use std::borrow::Borrow;

use chrono::NaiveDate;
use uuid::Uuid;

use crate::{
    accn::{Accn, AccnId, AccnMut, AccnStore, ContactId},
    journal::{Booking, Posting},
    valuable::{Money, Valuable},
};

#[allow(unused_variables)]
pub(crate) trait BuildBook {
    fn from_booking(booking: Booking) -> Self;
    fn with_posting(&mut self, accn: Accn, money: Option<Money>) -> &mut Self;

    fn with_moneys(&mut self, accn: Accn, moneys: impl IntoIterator<Item = Money>) -> &mut Self {
        for money in moneys.into_iter() {
            self.with_posting(accn.clone(), Some(money));
        }
        self
    }

    fn with_tag<'a>(
        &mut self,
        accns: &mut AccnStore,
        tag_name: &str,
        args: impl Iterator<Item = impl AsRef<str>>,
    ) -> &mut Self {
        self
    }

    fn into_booking_with(self, accns: &mut AccnStore) -> Booking;

    fn parse_accn<'a>(
        &mut self,
        accns: &'a mut AccnStore,
        names: impl IntoIterator<Item = impl Borrow<str>>,
    ) -> AccnMut<'a> {
        let mut names = names.into_iter();
        let mut accn = accns.root(names.next().unwrap().borrow()).unwrap().id();

        for name in names {
            let name: &str = name.borrow();
            name.strip_prefix('@').inspect(|name| {
                accns.add_contact(name);
            });
            accn = accns.accn_mut(accn).child_entry(name).or_open().id();
        }

        accns.accn_mut(accn)
    }
}

pub(crate) struct NoExtension {
    date: NaiveDate,
    desc: String,
    payee: ContactId,
    postings: Vec<Posting>,
    inferred_posting: Option<AccnId>,
}

impl BuildBook for NoExtension {
    fn from_booking(booking: Booking) -> Self {
        Self {
            date: booking.date,
            desc: booking.desc,
            payee: booking.payee,
            postings: booking.postings,
            inferred_posting: None,
        }
    }

    fn with_posting(&mut self, accn: Accn, money: Option<Money>) -> &mut Self {
        match money {
            Some(money) => self.postings.push(Posting {
                accn: accn.id(),
                money,
            }),
            None => self.inferred_posting = Some(accn.id()),
        };
        self
    }

    fn into_booking_with(mut self, _: &mut AccnStore) -> Booking {
        self.postings.extend(
            self.inferred_posting
                .map(|accn| {
                    self.inbalance()
                        .into_moneys()
                        .map(move |money| Posting { accn, money })
                })
                .into_iter()
                .flatten(),
        );

        Booking {
            id: Uuid::new_v4(),
            date: self.date,
            desc: self.desc,
            payee: self.payee,
            postings: self.postings,
        }
    }
}

impl NoExtension {
    fn inbalance(&self) -> Valuable {
        self.postings.iter().map(|p| -p.money.clone()).sum()
    }
}

macro_rules! allow_extensions {
    () => {
        NoExtension
    };

    ($name:ty) => {
        paste::paste! {
            $name<NoExtension>
        }
    };

    ($head:ty, $($tail:ty),+) => {
        paste::paste! {
            $head<allow_extensions!($($tail),+)>
        }
    };
}
