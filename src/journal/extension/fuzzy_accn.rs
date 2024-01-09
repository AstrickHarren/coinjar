use std::borrow::Borrow;

use itertools::Itertools;

use crate::{
    accn::{query::AccnQuery, Accn, AccnMut, AccnStore},
    journal::Booking,
};

use super::BuildBook;

pub(crate) struct FuzzyAccn<B: BuildBook> {
    inner: B,
    enabled: bool,

    recursive: bool,
    root: bool,
    adds_accn: bool,
}

impl<B: BuildBook> BuildBook for FuzzyAccn<B> {
    fn from_booking(booking: Booking) -> Self {
        Self {
            inner: B::from_booking(booking),
            enabled: false,

            recursive: false,
            root: false,
            adds_accn: false,
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
            self.enabled = true;
            for arg in args {
                match arg.as_ref() {
                    "recursive" | "deep" => self.recursive = true,
                    "root" | "root_only" => self.root = true,
                    "adds_accn" => self.adds_accn = true,
                    _ => panic!("unknown argument for tag 'fuzzy_accn': {}", arg.as_ref()),
                }
            }
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
        if !self.enabled {
            return self.inner.parse_accn(accns, names);
        }

        let id = names
            .into_iter()
            .exactly_one()
            .map(|name| {
                accns
                    .query(AccnQuery::NameIgnoreCase(name.borrow().to_string()))
                    .into_iter()
                    .exactly_one()
                    .unwrap_or_else(|_| panic!("accn not found: {}", name.borrow()))
                    .id()
            })
            .unwrap_or_else(|mut iter| match (self.recursive, self.root) {
                (true, _) => deep_parse_accn(iter, accns, self.adds_accn)
                    .unwrap_or_else(|iter| self.inner.parse_accn(accns, iter).id()),
                (false, true) => {
                    let root = fuzzy_root(accns, iter.next().unwrap().borrow())
                        .name()
                        .to_string();
                    let iter = std::iter::once(root).chain(iter.map(|s| s.borrow().to_string()));
                    self.inner.parse_accn(accns, iter).id()
                }
                (false, false) => self.inner.parse_accn(accns, iter).id(),
            });

        accns.accn_mut(id)
    }
}

fn deep_parse_accn(
    iter: impl Iterator<Item = impl Borrow<str>>,
    accns: &mut AccnStore,
    adds_accn: bool,
) -> Result<uuid::Uuid, impl Iterator<Item: Borrow<str>>> {
    // TODO: remove collection here
    let names = iter.map(|s| s.borrow().to_string()).collect_vec();
    let mut iter = names.clone().into_iter();

    let root = iter.next().unwrap();
    let root = fuzzy_root(accns, &root).id();

    let accn = iter.try_fold(root, |accn, name| {
        name.strip_prefix('@').inspect(|name| {
            accns.add_contact(name);
        });

        let fuzzy = accns
            .accn_mut(accn)
            .as_ref()
            .children()
            .filter(|child| child.name().to_lowercase().contains(&name.to_lowercase()))
            .map(|child| child.id())
            .exactly_one()
            .ok();

        fuzzy.or_else(|| {
            adds_accn.then(|| {
                accns
                    .accn_mut(accn)
                    .child_entry(name.to_string())
                    .or_open()
                    .id()
            })
        })
    });

    accn.ok_or_else(|| names.into_iter().map(|s| s))
}

fn fuzzy_root<'a>(accns: &'a mut AccnStore, root: &str) -> Accn<'a> {
    let accn = accns
        .roots()
        .filter(|accn| accn.name().contains(&root.to_lowercase()))
        .exactly_one()
        .unwrap_or_else(|_| panic!("root accn not found, got {}", root));
    accn
}
