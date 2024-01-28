pub(crate) mod entry;

use std::{collections::HashMap, fmt::Display};

use itertools::Itertools;
use uuid::Uuid;

pub(crate) use self::entry::{AccnEntry, AccnEntryMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) struct Accn {
    id: Uuid,
}

impl Accn {
    // WARNING: This should never be public, this way Accn can be used as a query key without check
    fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }

    pub(crate) fn into_accn_mut(self, tree: &mut AccnTree) -> AccnEntryMut {
        tree.accn_mut(self)
    }

    pub(crate) fn into_accn(self, tree: &AccnTree) -> AccnEntry {
        tree.accn(self)
    }
}

#[derive(Debug)]
struct AccnData {
    name: String,
    parent: Option<Accn>,
}

#[derive(Debug)]
pub(crate) struct AccnTree {
    root: Accn,
    accns: HashMap<Accn, AccnData>,
}

impl AccnTree {
    pub(crate) fn new() -> Self {
        let root = Accn::default();
        let mut accns = HashMap::new();
        accns.insert(
            root,
            AccnData {
                name: "root".to_string(),
                parent: None,
            },
        );
        let mut ret = Self { root, accns };

        ret.open_accn(root, "asset");
        ret.open_accn(root, "liability");
        ret.open_accn(root, "equity");
        ret.open_accn(root, "income");
        ret.open_accn(root, "expense");

        ret
    }

    pub(crate) fn root(&self) -> AccnEntry {
        self.accn(self.root)
    }

    pub(crate) fn root_mut(&mut self) -> AccnEntryMut {
        self.accn_mut(self.root)
    }

    pub(crate) fn expense(&self) -> AccnEntry {
        self.root().child("expense").unwrap()
    }

    pub(crate) fn income(&self) -> AccnEntry {
        self.root().child("income").unwrap()
    }

    fn open_accn(&mut self, parent: Accn, name: &str) -> Accn {
        let accn = Accn::new();
        self.accns.insert(
            accn,
            AccnData {
                name: name.to_string(),
                parent: Some(parent),
            },
        );
        accn
    }

    fn accn(&self, accn: Accn) -> AccnEntry {
        AccnEntry { accn, tree: self }
    }

    fn accn_mut(&mut self, accn: Accn) -> AccnEntryMut {
        AccnEntryMut { accn, tree: self }
    }

    fn accns(&self) -> impl Iterator<Item = AccnEntry> {
        self.accns.keys().copied().map(move |accn| self.accn(accn))
    }

    /// Return the AccnEntry for the given name, if it exists and unique.
    pub(crate) fn by_name_unique<'a, 'b>(
        &'a self,
        name: &'b str,
    ) -> Result<AccnEntry<'a>, impl Iterator<Item = AccnEntry<'a>> + 'b>
    where
        'a: 'b,
    {
        self.accns()
            .filter(move |accn| accn.name() == name)
            .exactly_one()
    }

    /// Takes a fuzzy input as `ex:common:food` and returns every accn that
    /// has all of its nearest ancestors with a name that contains the input.
    /// For example, `ex:common:food` would return `expense:common:food` and
    /// `asset:extra:common:food`
    pub(crate) fn by_name_fuzzy<'a>(
        &'a self,
        name: impl AccnPath<'a>,
    ) -> impl Iterator<Item = AccnEntry<'_>> + '_ {
        fn fuzzy_match(matcher: &str, matchee: &str) -> bool {
            matcher
                .to_lowercase()
                .contains(matchee.to_lowercase().as_str())
        }

        let parts = name.accn_path().collect_vec();
        let fuzzy = self
            .root()
            .traverse(
                vec![],
                move |st, accn| try {
                    st.push(accn.name());
                    (st.len() >= parts.len()).then_some(())?;
                    let accn = st
                        .iter()
                        .skip(st.len().saturating_sub(parts.len()))
                        .zip(parts.iter())
                        .all(|(st, pt)| fuzzy_match(st, pt))
                        .then_some(accn)?;
                    (accn != self.root()).then_some(accn)? // skip root
                },
                |st, _| {
                    st.pop();
                    None
                },
            )
            .flatten();

        fuzzy
    }
}

impl Display for AccnTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.root().fmt_proper_descendent(f)
    }
}

pub(crate) trait AccnPath<'a> {
    fn accn_path(self) -> impl Iterator<Item = &'a str>;
}

impl<'a> AccnPath<'a> for &'a str {
    fn accn_path(self) -> impl Iterator<Item = &'a str> {
        self.split(':')
    }
}

impl<'a> AccnPath<'a> for &Vec<&'a str> {
    fn accn_path(self) -> impl Iterator<Item = &'a str> {
        self.iter().copied()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_by_name_fuzzy() {
        let mut tree = AccnTree::new();
        tree.root_mut()
            .or_open_child("a")
            .or_open_child("aa")
            .or_open_child("aab")
            .or_open_child("aaab")
            .or_open_child("b")
            .or_open_child("ba")
            .or_open_child("bab")
            .or_open_child("baab");

        let entry = tree.by_name_fuzzy("a:a").map(|e| e.name()).collect_vec();
        assert_eq!(entry, vec!["aa", "aab", "aaab", "bab", "baab"]);
    }

    #[test]
    fn test_by_name_fuzzy_root() {
        let tree = AccnTree::new();
        let entry = tree.by_name_fuzzy("r:aasdf");
        assert_eq!(entry.count(), 0);
    }
}
