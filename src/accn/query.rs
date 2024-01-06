use std::collections::HashSet;

use itertools::Itertools;

use super::{Accn, AccnStore};

#[derive(Debug, Default, Clone)]
pub(super) enum AccnQuery {
    #[default]
    All,
    Name(String),
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AccnUnion<'a, T>
where
    T: Iterator<Item = Accn<'a>>,
{
    accns: T,
}

impl<'a, T> IntoIterator for AccnUnion<'a, T>
where
    T: Iterator<Item = Accn<'a>>,
{
    type Item = Accn<'a>;
    type IntoIter = T;

    fn into_iter(self) -> Self::IntoIter {
        self.accns
    }
}

impl AccnQuery {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn name(self, name: impl ToString) -> Self {
        Self::Name(name.to_string())
    }
}

impl AccnStore {
    pub(super) fn query(&self, query: AccnQuery) -> AccnUnion<Box<dyn Iterator<Item = Accn> + '_>> {
        match query {
            AccnQuery::Name(name) => AccnUnion {
                accns: Box::new(self.accns().filter(move |a| a.name().contains(&name))),
            },
            AccnQuery::All => AccnUnion {
                accns: Box::new(self.accns()),
            },
        }
    }
}

impl<'a, T> AccnUnion<'a, T>
where
    T: Iterator<Item = Accn<'a>>,
{
    pub(crate) fn elders(self) -> AccnUnion<'a, impl Iterator<Item = Accn<'a>>> {
        let mut accns = self.accns.collect_vec();
        let accn_ids: HashSet<_> = accns.iter().map(|a| a.id()).collect();

        accns.retain(|accn| {
            !accn
                .ancesters_exclusive()
                .any(|a| accn_ids.contains(&a.id()))
        });

        AccnUnion {
            accns: accns.into_iter(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::accn::tests::example_accn_store;

    use super::*;

    #[test]
    fn test_elders() {
        let store = example_accn_store();
        let query = AccnQuery::new().name("drinks".to_string());
        let vec = store
            .query(query.clone())
            .into_iter()
            .map(|a| a.abs_name())
            .collect_vec();
        dbg!(vec);

        let vec = store
            .query(query.clone())
            .elders()
            .into_iter()
            .map(|a| a.abs_name())
            .collect_vec();
        dbg!(vec);
    }
}
