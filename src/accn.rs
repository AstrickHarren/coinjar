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
struct AccnStore {
    root_accns: RootAccns,
    accn_data: HashMap<AccnId, AccnData>,
    contacts: HashMap<Uuid, ContactData>,
}

#[derive(Debug)]
struct Accn<'a> {
    id: AccnId,
    accn_store: &'a AccnStore,
}

macro_rules! root_accn {
    ($($name:ident),*) => {
        fn new() -> Self {
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

    fn abs_name(&self) -> String {
        self.ancesters()
            .map(|accn| accn.name().to_string())
            .collect_vec()
            .into_iter()
            .rev()
            .join("/")
    }
}

impl Into<AccnId> for Accn<'_> {
    fn into(self) -> AccnId {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let food = store.open_accn("food", Some(store.asset().into())).id;
        let drinks = store.open_accn("drinks", Some(food)).id;
        let beer = store.open_accn("beer", Some(drinks));
        assert_eq!(beer.abs_name(), "asset/food/drinks/beer");
    }
}
