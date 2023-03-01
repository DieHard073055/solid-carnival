use chrono::Utc;
use rust_decimal::prelude::Decimal;

#[derive(Debug, Clone, PartialEq)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderDirection {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderStatus {
    Pending,
    PartiallyFilled(u8),
    Filled,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Order {
    pub ts: i64,
    pub order_type: OrderType,
    pub direction: OrderDirection,
    pub pair: String,
    pub price: Option<Decimal>,
    pub qty: Decimal,
    pub status: OrderStatus,
}

impl Order {
    fn new(
        ts: i64,
        order_type: OrderType,
        direction: OrderDirection,
        pair: String,
        price: Option<Decimal>,
        qty: Decimal,
        status: OrderStatus,
    ) -> Self {
        Order {
            ts,
            order_type,
            direction,
            pair,
            price,
            qty,
            status,
        }
    }
    pub fn new_limit_buy(pair: &str, price: Decimal, qty: Decimal) -> Self {
        Order::new(
            Utc::now().timestamp(),
            OrderType::Limit,
            OrderDirection::Buy,
            String::from(pair),
            Some(price),
            qty,
            OrderStatus::Pending,
        )
    }
    pub fn new_limit_sell(pair: &str, price: Decimal, qty: Decimal) -> Self {
        Order::new(
            Utc::now().timestamp(),
            OrderType::Limit,
            OrderDirection::Sell,
            String::from(pair),
            Some(price),
            qty,
            OrderStatus::Pending,
        )
    }
    pub fn new_market_buy(pair: &str, price: Decimal, qty: Decimal) -> Self {
        Order::new(
            Utc::now().timestamp(),
            OrderType::Market,
            OrderDirection::Buy,
            String::from(pair),
            None,
            qty,
            OrderStatus::Pending,
        )
    }
    pub fn new_market_sell(pair: &str, price: Decimal, qty: Decimal) -> Self {
        Order::new(
            Utc::now().timestamp(),
            OrderType::Market,
            OrderDirection::Sell,
            String::from(pair),
            None,
            qty,
            OrderStatus::Pending,
        )
    }
}
