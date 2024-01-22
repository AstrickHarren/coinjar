pub(crate) mod entry;

use std::collections::HashMap;

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

    fn into_accn_mut(self, tree: &mut AccnTree) -> AccnEntryMut {
        tree.accn_mut(self)
    }

    fn into_accn(self, tree: &AccnTree) -> AccnEntry {
        tree.accn(self)
    }
}

#[derive(Debug)]
struct AccnData {
    name: String,
    parent: Option<Accn>,
}

enum AccnType {
    Asset,
    Liability,
    Equity,
    Income,
    Expense,
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
        Self { root, accns }
    }

    pub(crate) fn root(&self) -> AccnEntry {
        self.accn(self.root)
    }

    pub(crate) fn root_mut(&mut self) -> AccnEntryMut {
        self.accn_mut(self.root)
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
}
