use std::{collections::HashMap, fmt::Display};

use tabled::{settings::Style, Table};

pub(crate) trait DisplayTable {
    fn into_table(self) -> Table;
}

impl<K, V> DisplayTable for HashMap<K, V>
where
    K: Display,
    V: Display,
{
    fn into_table(self) -> Table {
        let mut table = Table::new(
            self.into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string())),
        );
        table.with(Style::modern());
        table
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table() {
        let mut map = HashMap::new();
        map.insert("a", 1);
        map.insert("b", 2);
        map.insert("c", 3);

        println!("{}", map.into_table());
    }
}
