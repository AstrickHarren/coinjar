use itertools::Itertools;
use paste::paste;
use std::{collections::HashMap, fmt::Display};
use uuid::Uuid;

pub(super) type AccnId = Uuid;
pub(super) type ContactId = Uuid;

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
pub(crate) struct AccnEntry<'a> {
    accn_store: &'a mut AccnStore,
    name: String,
    parent: AccnId,
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
            pub(crate) fn $name(&self) -> Accn {
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

        pub(crate) fn root(&self, name: &str) -> Option<Accn> {
            match name {
                $(
                    stringify!($name) => Some(self.$name()),
                )*
                _ => None,
            }
        }
    };
}

impl AccnStore {
    pub(crate) fn open_accn(&mut self, name: impl ToString, parent: Option<AccnId>) -> AccnMut {
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

    pub(crate) fn find_accn_mut(&mut self, name: &str) -> Option<AccnMut> {
        let accn_id = self
            .accn_data
            .values()
            .find(|data| data.name == name)
            .map(|data| data.id)
            .map(|id| AccnMut {
                id,
                accn_store: self,
            });

        accn_id
    }

    pub(crate) fn accn(&self, id: AccnId) -> Accn {
        Accn {
            id,
            accn_store: self,
        }
    }

    pub(crate) fn accn_mut(&mut self, id: AccnId) -> AccnMut {
        AccnMut {
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

    pub(crate) fn find_contact(&self, name: &str) -> Option<Contact> {
        self.contacts
            .values()
            .find(|contact| contact.name == name)
            .map(|contact| Contact {
                id: contact.id,
                accn_store: self,
            })
    }

    pub(crate) fn find_contact_mut(&mut self, name: &str) -> Option<ContactMut> {
        let contact_id = self
            .contacts
            .values()
            .find(|contact| contact.name == name)
            .map(|contact| contact.id)
            .map(|id| ContactMut {
                id,
                accn_store: self,
            });

        contact_id
    }

    pub(crate) fn contact(&self, id: ContactId) -> Contact {
        Contact {
            id,
            accn_store: self,
        }
    }

    pub(crate) fn contact_mut(&mut self, id: ContactId) -> ContactMut {
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

impl Display for AccnStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.accns()
            .map(|a| a.abs_name())
            .sorted()
            .format("\n")
            .fmt(f)
    }
}

impl Accn<'_> {
    pub(crate) fn ancesters(&self) -> impl Iterator<Item = Accn> + '_ {
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

    fn parent(&self) -> Option<Accn> {
        self.accn_store
            .accn_data
            .get(&self.id)
            .and_then(|data| data.parent)
            .map(|id| Accn {
                id,
                accn_store: self.accn_store,
            })
    }

    pub(crate) fn name(&self) -> &str {
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

    fn children(&self) -> impl Iterator<Item = Accn> {
        self.accn_store
            .accns()
            .filter(|accn| accn.parent().map(|p| p.id()) == Some(self.id))
    }
}

impl<'a> AccnMut<'a> {
    fn as_ref(&self) -> Accn<'_> {
        Accn {
            id: self.id,
            accn_store: self.accn_store,
        }
    }

    pub(crate) fn id(&self) -> AccnId {
        self.id
    }

    pub(crate) fn open_child_accn(&mut self, name: impl ToString) -> AccnMut {
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

    pub(crate) fn child_entry(&mut self, name: impl ToString) -> AccnEntry {
        AccnEntry {
            accn_store: self.accn_store,
            name: name.to_string(),
            parent: self.id,
        }
    }

    pub(crate) fn child(&mut self, name: &str) -> Option<AccnMut> {
        let child = self
            .as_ref()
            .children()
            .find(|a| a.name() == name)
            .map(|a| a.id());
        child.map(|id| AccnMut {
            id,
            accn_store: self.accn_store,
        })
    }
}

impl AccnEntry<'_> {
    pub(crate) fn or_open(&mut self) -> AccnMut {
        let id = self
            .accn_store
            .accn_mut(self.parent)
            .child(&self.name)
            .map(|accn| accn.id())
            .unwrap_or_else(|| {
                self.accn_store
                    .open_accn(&self.name, Some(self.parent))
                    .id()
            });

        AccnMut {
            id,
            accn_store: self.accn_store,
        }
    }
}

impl Contact<'_> {
    pub(crate) fn name(&self) -> &str {
        &self.accn_store.contacts[&self.id].name
    }

    pub(crate) fn id(&self) -> ContactId {
        self.id
    }
}

macro_rules! impl_into {
    ($name:ident : $type:ty; $($target:ty),*) => {
        $(
        impl Into<$type> for $target {
            fn into(self) -> $type {
                self.$name
            }
        }
        )*
    };
}

impl_into!(id: ContactId; Contact<'_>, ContactMut<'_>, &Contact<'_>, &ContactMut<'_>);

impl ContactMut<'_> {
    pub(crate) fn as_ref(&self) -> Contact<'_> {
        Contact {
            id: self.id,
            accn_store: self.accn_store,
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.accn_store.contacts[&self.id].name
    }

    pub(crate) fn id(&self) -> ContactId {
        self.id
    }

    pub(crate) fn make_accns(&mut self) {
        let name = self.name().to_string();
        let name = "@".to_string() + &name;
        self.accn_store
            .asset_mut()
            .child_entry(&name)
            .or_open()
            .child_entry("receivable")
            .or_open();

        self.accn_store
            .liability_mut()
            .child_entry(&name)
            .or_open()
            .child_entry("payable")
            .or_open();
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
