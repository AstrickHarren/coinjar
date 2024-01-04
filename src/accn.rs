use itertools::Itertools;
use paste::paste;
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

#[derive(Debug)]
pub(crate) struct AccnMut<'a> {
    id: AccnId,
    accn_store: &'a mut AccnStore,
}

#[derive(Debug)]
pub(crate) struct Contact<'a> {
    id: ContactId,
    accn_store: &'a AccnStore,
}

#[derive(Debug)]
pub(crate) struct ContactMut<'a> {
    id: ContactId,
    accn_store: &'a mut AccnStore,
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

            paste! {
                fn [<$name _mut>](&mut self) -> AccnMut {
                    AccnMut{
                        id: self.root_accns.$name,
                        accn_store: self,
                    }
                }
            }
        )*
    };
}

impl AccnStore {
    fn open_accn(&mut self, name: impl ToString, parent: Option<AccnId>) -> AccnMut {
        let id = Uuid::new_v4();
        let accn_data = AccnData {
            id,
            name: name.to_string(),
            parent: parent.map(|id| id.into()),
        };
        self.accn_data.insert(id, accn_data);
        AccnMut {
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

    pub(crate) fn add_contact(&mut self, name: impl ToString) -> ContactMut {
        let id = Uuid::new_v4();
        let name = name.to_string();

        let contact = ContactData {
            id,
            name: name.clone(),
        };
        self.contacts.insert(id, contact);

        ContactMut {
            id,
            accn_store: self,
        }
    }

    root_accn!(asset, liability, income, expense, equity);

    fn accns(&self) -> impl Iterator<Item = Accn> + '_ {
        self.accn_data.keys().map(move |&id| Accn {
            id,
            accn_store: self,
        })
    }
}

impl Accn<'_> {
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

impl AccnMut<'_> {
    fn as_ref(&self) -> Accn<'_> {
        Accn {
            id: self.id,
            accn_store: self.accn_store,
        }
    }

    fn id(&self) -> AccnId {
        self.id
    }

    fn open_child_accn(&mut self, name: impl ToString) -> AccnMut {
        let id = Uuid::new_v4();
        let accn_data = AccnData {
            id,
            name: name.to_string(),
            parent: Some(self.id),
        };
        self.accn_store.accn_data.insert(id, accn_data);
        AccnMut {
            id,
            accn_store: self.accn_store,
        }
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

    pub(crate) fn example_accn_store() -> AccnStore {
        let mut store = AccnStore::new();

        let mut expense = store.expense_mut();
        let mut food = expense.open_child_accn("food");
        let mut drinks = food.open_child_accn("drinks");
        drinks.open_child_accn("beer");
        drinks.open_child_accn("wine");
        drinks.open_child_accn("chips");

        let mut income = store.income_mut();
        income.open_child_accn("salary");

        store.add_contact("Alice");
        store.add_contact("Bob");

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
        assert_eq!(beer.as_ref().abs_name(), "asset/food/drinks/beer");
    }

    #[test]
    fn test_example_accn_store() {
        let store = example_accn_store();
        let ret = store
            .accns()
            .sorted_by_key(|accn| accn.abs_name())
            .format_with("\n", |accn, f| f(&format_args!("{}", accn.abs_name())));
        println!("{}", ret);
    }
}
