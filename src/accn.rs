use std::collections::HashMap;
use uuid::Uuid;

type AccnId = Uuid;
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
            fn $name(&self) -> AccnId {
                self.root_accns.$name
            }
        )*
    };
}

impl AccnStore {
    fn open_accn(&mut self, name: String, parent: Option<AccnId>) -> Accn {
        let id = Uuid::new_v4();
        let accn_data = AccnData { id, name, parent };
        self.accn_data.insert(id, accn_data);
        Accn {
            id,
            accn_store: self,
        }
    }

    root_accn!(asset, liability, income, expense, equity);
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
}
