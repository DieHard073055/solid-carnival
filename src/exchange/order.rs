use chrono::Utc;
use rust_decimal::prelude::Decimal;
use std::sync::atomic::{AtomicU64, Ordering};

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

// Create a static atomic counter for order IDs
static ORDER_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq)]
pub struct Order {
    pub id: u64,
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
        id: u64,
        ts: i64,
        order_type: OrderType,
        direction: OrderDirection,
        pair: String,
        price: Option<Decimal>,
        qty: Decimal,
        status: OrderStatus,
    ) -> Self {
        Order {
            id,
            ts,
            order_type,
            direction,
            pair,
            price,
            qty,
            status,
        }
    }
    pub fn new_order(
        pair: &str,
        price: Option<Decimal>,
        qty: Decimal,
        direction: OrderDirection,
        order_type: OrderType,
    ) -> Self {
        let id = ORDER_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let ts = Utc::now().timestamp();
        Order::new(
            id,
            ts,
            order_type,
            direction,
            String::from(pair),
            price,
            qty,
            OrderStatus::Pending,
        )
    }
    pub fn new_limit_buy(pair: &str, price: Decimal, qty: Decimal) -> Self {
        Order::new_order(
            pair,
            Some(price),
            qty,
            OrderDirection::Buy,
            OrderType::Limit,
        )
    }
    pub fn new_limit_sell(pair: &str, price: Decimal, qty: Decimal) -> Self {
        Order::new_order(
            pair,
            Some(price),
            qty,
            OrderDirection::Sell,
            OrderType::Limit,
        )
    }
    pub fn new_market_buy(pair: &str, price: Decimal, qty: Decimal) -> Self {
        Order::new_order(
            pair,
            Some(price),
            qty,
            OrderDirection::Buy,
            OrderType::Market,
        )
    }
    pub fn new_market_sell(pair: &str, price: Decimal, qty: Decimal) -> Self {
        Order::new_order(
            pair,
            Some(price),
            qty,
            OrderDirection::Sell,
            OrderType::Market,
        )
    }
    pub fn filled(&mut self) {
        self.status = OrderStatus::Filled;
    }
}
