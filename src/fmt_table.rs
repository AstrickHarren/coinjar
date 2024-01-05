use std::{collections::HashMap, fmt::Display, iter::once};

use tabled::{settings::Style, tables::IterTable};

pub(crate) trait DisplayTable {
    fn fmt_table(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;

    fn as_table(&self) -> Tabled<&Self> {
        Tabled { data: self }
    }
}

impl<T: DisplayTable> DisplayTable for &T {
    fn fmt_table(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (*self).fmt_table(f)
    }
}

pub(crate) struct Tabled<T> {
    data: T,
}

impl<T> Display for Tabled<T>
where
    T: DisplayTable,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt_table(f)
    }
}

impl<K, V> DisplayTable for HashMap<K, V>
where
    K: Display,
    V: Display,
{
    fn fmt_table(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        IterTable::new(
            self.iter()
                .map(|(k, v)| once(k.to_string()).chain(once(v.to_string()))),
        )
        .with(Style::modern())
        .fmt(f)
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

        println!("{}", map.as_table());
    }
}
