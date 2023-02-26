use rust_decimal::prelude::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone)]
pub struct Transaction {
    ts: i64,
    symbol: String,
    price: Decimal,
    qty: Decimal,
}

impl Transaction{
    pub fn new(ts: i64,
               symbol: String,
               price: Decimal,
               qty: Decimal) -> Self{
        Transaction {
            ts,
            symbol,
            price,
            qty,
        }
    }
    pub fn get_ts(&self) -> &i64 {
        &self.ts
    }
    pub fn get_symbol(&self) -> &String {
        &self.symbol
    }
    pub fn get_price(&self) -> &Decimal {
        &self.price
    }
    pub fn get_qty(&self) -> &Decimal {
        &self.qty
    }
}