use std::borrow::Borrow;

use itertools::Itertools;

use crate::{
    accn::{query::AccnQuery, AccnMut, AccnStore},
    journal::Booking,
};

use super::BuildBook;

pub(crate) struct FuzzyAccn<B: BuildBook> {
    inner: B,
    enabled: bool,
}

impl<B: BuildBook> BuildBook for FuzzyAccn<B> {
    fn from_booking(booking: Booking) -> Self {
        Self {
            inner: B::from_booking(booking),
            enabled: false,
        }
    }

    fn with_posting(
        &mut self,
        accn: crate::accn::Accn,
        money: Option<crate::valuable::Money>,
    ) -> &mut Self {
        self.inner.with_posting(accn, money);
        self
    }

    fn into_booking_with(self, accns: &mut AccnStore) -> Booking {
        self.inner.into_booking_with(accns)
    }

    fn with_tag<'a>(
        &mut self,
        accns: &mut AccnStore,
        tag_name: &str,
        args: impl Iterator<Item = impl AsRef<str>>,
    ) -> &mut Self {
        if tag_name == "fuzzy_accn" {
            debug_assert!(args.count() == 0);
            self.enabled = true;
        } else {
            self.inner.with_tag(accns, tag_name, args);
        }

        self
    }

    fn parse_accn<'a>(
        &mut self,
        accns: &'a mut AccnStore,
        names: impl IntoIterator<Item = impl Borrow<str>>,
    ) -> AccnMut<'a> {
        dbg!(self.enabled);
        if !self.enabled {
            return self.inner.parse_accn(accns, names);
        }

        let id = names
            .into_iter()
            .exactly_one()
            .map(|name| {
                accns
                    .query(AccnQuery::Name(name.borrow().to_string()))
                    .into_iter()
                    .exactly_one()
                    .unwrap_or_else(|_| panic!("accn not found: {}", name.borrow()))
                    .id()
            })
            .unwrap_or_else(|iter| {
                // TODO: remove collection here
                let names = iter.map(|s| s.borrow().to_string()).collect_vec();
                dbg!(&names);
                let mut iter = names.clone().into_iter();

                let root = iter.next().unwrap();
                let mut accn = accns
                    .roots()
                    .filter(|accn| accn.name().contains(&root.to_lowercase()))
                    .exactly_one()
                    .unwrap_or_else(|_| panic!("root accn not found, got {}", root))
                    .id();

                for name in iter {
                    let name = name
                        .strip_prefix('@')
                        .inspect(|name| {
                            accns.add_contact(name);
                        })
                        .unwrap_or(&name);

                    let fuzzy = accns
                        .accn_mut(accn)
                        .as_ref()
                        .children()
                        .filter(|child| child.name().to_lowercase().contains(&name.to_lowercase()))
                        .map(|child| child.id())
                        .exactly_one()
                        .map_err(|_| ());

                    accn = fuzzy.unwrap_or_else(|_| {
                        accns
                            .accn_mut(accn)
                            .child_entry(name.to_string())
                            .or_open()
                            .id()
                    });
                }

                accn
            });

        accns.accn_mut(id)
    }
}
