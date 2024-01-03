use itertools::Itertools;
use std::collections::HashMap;
use uuid::Uuid;

pub(super) type AccnId = Uuid;
type ContactId = Uuid;

#[derive(Debug)]
struct AccnData {
    id: AccnId,
    name: String,
    parent: Option<AccnId>,
}

#[derive(Debug)]
struct RootAccns {
    asset: AccnId,
    liability: AccnId,
    income: AccnId,
    expense: AccnId,
    equity: AccnId,
}

#[derive(Debug, Default)]
struct ContactData {
    id: ContactId,
    name: String,
}

#[derive(Debug)]
pub(super) struct AccnStore {
    root_accns: RootAccns,
    accn_data: HashMap<AccnId, AccnData>,
    contacts: HashMap<Uuid, ContactData>,
}

#[derive(Debug)]
pub(crate) struct Accn<'a> {
    id: AccnId,
    accn_store: &'a AccnStore,
}

macro_rules! root_accn {
    ($($name:ident),*) => {
        pub(crate) fn new() -> Self {
            let root_accns = RootAccns {
                $($name: Uuid::new_v4(),)*
            };
            let mut accn_data = HashMap::new();
            $(
                let id = root_accns.$name;
                let name = stringify!($name).to_string();
                let data = AccnData { id, name, parent: None };
                accn_data.insert(id, data);
            )*
            Self { root_accns, accn_data, contacts: Default::default()}
        }

        $(
            fn $name(&self) -> Accn {
                Accn{
                    id: self.root_accns.$name,
                    accn_store: self,
                }
            }
        )*
    };
}

impl AccnStore {
    fn open_accn(&mut self, name: impl ToString, parent: Option<AccnId>) -> Accn {
        let id = Uuid::new_v4();
        let accn_data = AccnData {
            id,
            name: name.to_string(),
            parent: parent.map(|id| id.into()),
        };
        self.accn_data.insert(id, accn_data);
        Accn {
            id,
            accn_store: self,
        }
    }

    pub(crate) fn find_accn(&self, name: &str) -> Option<Accn> {
        self.accn_data
            .values()
            .find(|data| data.name == name)
            .map(|data| Accn {
                id: data.id,
                accn_store: self,
            })
    }

    pub(crate) fn accn(&self, id: AccnId) -> Accn {
        Accn {
            id,
            accn_store: self,
        }
    }

    root_accn!(asset, liability, income, expense, equity);
}

impl<'a> Accn<'a> {
    fn ancesters(&self) -> impl Iterator<Item = Accn> + '_ {
        std::iter::successors(Some(self.id), |&id| {
            self.accn_store
                .accn_data
                .get(&id)
                .and_then(|data| data.parent)
        })
        .map(|id| Accn {
            id,
            accn_store: self.accn_store,
        })
    }

    fn name(&self) -> &str {
        &self.accn_store.accn_data[&self.id].name
    }

    pub(crate) fn abs_name(&self) -> String {
        self.ancesters()
            .map(|accn| accn.name().to_string())
            .collect_vec()
            .into_iter()
            .rev()
            .join("/")
    }

    pub(super) fn id(&self) -> AccnId {
        self.id
    }
}

impl Into<AccnId> for Accn<'_> {
    fn into(self) -> AccnId {
        self.id
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn test_accn_store() -> AccnStore {
        let mut store = AccnStore::new();
        let food = store.open_accn("food", Some(store.asset().into())).id();
        let drinks = store.open_accn("drinks", Some(food)).id();
        let _beer = store.open_accn("beer", Some(drinks));
        let _wine = store.open_accn("wine", Some(drinks));
        let _chips = store.open_accn("chips", Some(drinks));
        let _salary = store.open_accn("salary", Some(store.income().into()));
        let _rent = store.open_accn("rent", Some(store.expense().into()));
        let mut contacts = HashMap::new();
        contacts.insert(
            Uuid::new_v4(),
            ContactData {
                id: Uuid::new_v4(),
                name: "John Doe".to_string(),
            },
        );
        contacts.insert(
            Uuid::new_v4(),
            ContactData {
                id: Uuid::new_v4(),
                name: "Jane Doe".to_string(),
            },
        );
        store.contacts = contacts;
        store
    }

    #[test]
    fn test_new_accn() {
        let store = AccnStore::new();
        assert!(store.accn_data.len() == 5);
        macro_rules! assert_root {
            ($($name:ident),*) => {
                $(
                    assert!(store.accn_data.contains_key(&store.root_accns.$name));
                )*
            };
        }
        assert_root!(asset, liability, income, expense, equity);
    }

    #[test]
    fn test_abs_name() {
        let mut store = AccnStore::new();
        let food = store.open_accn("food", Some(store.asset().into())).id();
        let drinks = store.open_accn("drinks", Some(food)).id();
        let beer = store.open_accn("beer", Some(drinks));
        assert_eq!(beer.abs_name(), "asset/food/drinks/beer");
    }
}
