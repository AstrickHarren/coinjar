use std::sync::Arc;

#[derive(Debug)]
struct Currency {
    name: Option<&'static str>,
    symbol: Option<&'static str>,
    code: Arc<&'static str>,
}

#[derive(Debug)]
struct Money {
    amount: f32,
    currency: Currency,
}

struct Valuable {
    moneys: Vec<Money>,
}
