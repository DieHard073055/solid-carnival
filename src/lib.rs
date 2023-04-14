pub mod exchange;

use thiserror::Error;

use std::collections::HashMap;
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;
use crate::exchange::exchange::{Exchange, ExchangeError};
use crate::exchange::price_feed::PriceFeed;

#[derive(Debug, Clone, Error)]
pub enum ExchangesError {
    #[error("No exchange with the given exchange_id")]
    InvalidExchangeId,
}

struct Exchanges {
    exchanges: HashMap<String, Exchange>
}
impl Exchanges {
    fn create_new_exchange(&mut self) -> &str {
        let mut exchange = Exchange::new();

        let exchange_id = exchange.get_instance_id();
        self.exchanges.insert(exchange_id.to_string(), exchange);
        exchange_id
    }
    fn add_capital(&mut self, instance_id: &str, symbol: &str, amount: Decimal) -> Result<(), ExchangeError>{
        if let Some(exchange) = self.exchanges.get_mut(instance_id) {
            exchange.with_capital(vec![(symbol.to_string(), amount)])?
        }
        Ok(())
    }
    fn add_price_feed(&mut self, instance_id: &str, symbol: &str, interval: &str, limit: i32){
        if let Some(exchange) = self.exchanges.get_mut(instance_id) {
            exchange.with_price_feed(symbol.to_string(), interval.to_string(), limit);
        }
    }
    fn tick(&mut self, instance_id: &str){
        if let Some(exchange) = self.exchanges.get_mut(instance_id) {
            exchange.tick();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::exchange::order::{Order, OrderDirection, OrderType};
    use chrono::format::Numeric::Timestamp;
    use rust_decimal_macros::dec;
    use std::task::Context;
    use rust_decimal::Decimal;
    use rust_decimal::prelude::ToPrimitive;
    use uuid::Uuid;
    use std::io::Write;

    fn fill_from_str(mut bytes: &mut [u8], s: &str) {
        bytes.write(s.as_bytes()).unwrap();
    }
    fn create_order_uuid(pair: &str, price: Decimal, qty: Decimal, direction: OrderDirection, order_type: OrderType) {
        let order = Order::new_order(
            pair,
            Some(price),
            qty,
            direction,
            order_type
        );
        let order_direction = if order.direction == OrderDirection::Buy {0u8} else {1u8};
        let order_type = if order.order_type == OrderType::Limit {0u8} else {1u8};
        let order_price = order.price.unwrap_or(dec!(0));
        let order_details = format!("{:}{:}{:}{:}{:}{:}", order.ts, order.pair, order_price, order.qty, order_direction, order_type);
        let mut order_details_bytes: [u8; 16] = [0; 16];
        fill_from_str(&mut order_details_bytes, order_details.as_str());
        println!("{:}", order_details);
        println!("{:?}", order_details.as_bytes());
        println!("{:?}", order_details_bytes);
        let order_id = Uuid::from_slice(&order_details_bytes).unwrap();
        println!("order_id : {:}", order_id );
    }
    #[test]
    fn test_gen_uuid() {
        create_order_uuid("BTCUSDT", dec!(19000), dec!(1), OrderDirection::Buy, OrderType::Limit);
        create_order_uuid("BTCUSDT", dec!(19000), dec!(1), OrderDirection::Buy, OrderType::Limit);
        create_order_uuid("BTCUSDT", dec!(19500), dec!(1), OrderDirection::Buy, OrderType::Limit);
        create_order_uuid("BTCUSDT", dec!(19000), dec!(0.1), OrderDirection::Buy, OrderType::Limit);
        create_order_uuid("BTCUSDT", dec!(19000), dec!(0.1), OrderDirection::Sell, OrderType::Limit);
        create_order_uuid("BTCUSDT", dec!(19000), dec!(1), OrderDirection::Buy, OrderType::Market);

        // println!("order_id : {:}", order_id );
    }
}
