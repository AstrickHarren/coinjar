use std::fmt::Display;

use itertools::Itertools;

use super::*;
#[derive(Clone, Copy, Debug)]
pub(crate) struct AccnEntry<'a> {
    pub(super) accn: Accn,
    pub(super) tree: &'a AccnTree,
}

impl Display for AccnEntry<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.ancestors()
            .collect_vec()
            .into_iter()
            .rev()
            .skip(1) // skip root
            .map(|accn| accn.name())
            .join(":")
            .fmt(f)
    }
}

impl PartialEq for AccnEntry<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.accn == other.accn
    }
}

impl<'a> AccnEntry<'a> {
    fn children(self) -> impl Iterator<Item = AccnEntry<'a>> {
        self.tree
            .accns
            .iter()
            .filter(move |(_, data)| data.parent == Some(self.accn))
            .map(move |(accn, _)| accn.into_accn(self.tree))
    }

    fn ancestors(self) -> impl Iterator<Item = AccnEntry<'a>> {
        std::iter::successors(Some(self), move |accn| accn.parent())
    }

    fn parent(self) -> Option<AccnEntry<'a>> {
        println!("parent of {}", self.data().name);
        let parent = self.data().parent?;
        dbg!(self.accn, self.data(), parent);
        Some(parent.into_accn(self.tree))
    }

    fn data(self) -> &'a AccnData {
        &self.tree.accns[&self.accn]
    }

    fn child(self, name: &str) -> Option<AccnEntry<'a>> {
        self.children().find(move |child| child.name() == name)
    }

    fn name(self) -> &'a str {
        &self.tree.accns[&self.accn].name
    }

    fn as_mut(self, tree: &mut AccnTree) -> AccnEntryMut<'_> {
        AccnEntryMut {
            accn: self.accn,
            tree,
        }
    }

    pub(crate) fn id(self) -> Accn {
        self.accn
    }
}

pub(crate) struct AccnEntryMut<'a> {
    pub(super) accn: Accn,
    pub(super) tree: &'a mut AccnTree,
}

impl Display for AccnEntryMut<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<'a> AccnEntryMut<'a> {
    pub(crate) fn as_ref(&'a self) -> AccnEntry<'a> {
        AccnEntry {
            accn: self.accn,
            tree: self.tree,
        }
    }

    pub(crate) fn or_open_child(self, name: &str) -> AccnEntryMut<'a> {
        let child = self.as_ref().child(name);

        match child {
            Some(child) => child.accn.into_accn_mut(self.tree),
            None => self
                .tree
                .open_accn(self.accn, name)
                .into_accn_mut(self.tree),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn example_tree() -> AccnTree {
        let mut tree = AccnTree::new();
        tree.root_mut()
            .or_open_child("assets")
            .or_open_child("bank")
            .or_open_child("checking");
        tree
    }

    #[test]
    fn test_parent() {
        let tree = example_tree();
        let asset = tree.root().child("assets").unwrap();
        let bank = asset.child("bank").unwrap();
        let checking = bank.child("checking").unwrap();

        assert_eq!(checking.parent(), Some(bank));
        assert_eq!(bank.parent(), Some(asset));
        assert_eq!(asset.parent(), Some(tree.root()));
    }

    #[test]
    fn test_ancestor() {
        let example_tree = example_tree();
        let checking: Option<_> = try {
            example_tree
                .root()
                .child("assets")?
                .child("bank")?
                .child("checking")?
        };

        assert_eq!(checking.unwrap().ancestors().count(), 4);
    }

    #[test]
    fn test_display() {
        let example_tree = example_tree();
        let checking: Option<_> = try {
            example_tree
                .root()
                .child("assets")?
                .child("bank")?
                .child("checking")?
        };

        assert_eq!(checking.unwrap().to_string(), "assets:bank:checking");
    }
}
