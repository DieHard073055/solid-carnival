pub mod exchange;

use thiserror::Error;

use crate::exchange::exchange::{Exchange, ExchangeError};
use crate::exchange::price_feed::PriceFeed;
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use thiserror::__private::AsDynError;
use uuid::Uuid;

#[derive(Debug, Clone, Error)]
pub enum ExchangesError {
    #[error("No exchange with the given exchange_id")]
    InvalidExchangeId,
}

struct Exchanges {
    exchanges: HashMap<String, Exchange>,
}
impl Exchanges {
    pub fn new() -> Self {
        Self {
            exchanges: HashMap::new(),
        }
    }
    pub fn create_new_exchange(&mut self) -> String {
        let instance_id = Uuid::new_v4().hyphenated().to_string();
        self.exchanges.insert(instance_id.clone(), Exchange::new());
        instance_id
    }
    pub fn mut_unwrap_exchange_from_instance(
        &mut self,
        instance_id: &str,
    ) -> Result<&mut Exchange, ExchangesError> {
        if let Some(exchange) = self.exchanges.get_mut(instance_id) {
            Ok(exchange)
        } else {
            Err(ExchangesError::InvalidExchangeId)
        }
    }
    pub fn unwrap_exchange_from_instance(
        &self,
        instance_id: &str,
    ) -> Result<&Exchange, ExchangesError> {
        if let Some(exchange) = self.exchanges.get(instance_id) {
            Ok(exchange)
        } else {
            Err(ExchangesError::InvalidExchangeId)
        }
    }
    pub fn add_capital(
        &mut self,
        instance_id: &str,
        symbol: &str,
        amount: Decimal,
    ) -> Result<(), ExchangesError> {
        let exchange = self.mut_unwrap_exchange_from_instance(instance_id)?;
        exchange.with_capital(vec![(symbol.to_string(), amount)]);
        Ok(())
    }
    pub fn add_price_feed(
        &mut self,
        instance_id: &str,
        symbol: &str,
        interval: &str,
        limit: i32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let exchange = self.mut_unwrap_exchange_from_instance(instance_id)?;
        exchange.with_price_feed(symbol.to_string(), interval.to_string(), limit)?;
        Ok(())
    }
    pub fn tick(
        &mut self,
        instance_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let exchange = self.mut_unwrap_exchange_from_instance(instance_id)?;
        exchange.tick()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::exchange::order::OrderStatus;
    use crate::exchange::price_feed::BinanceKline;

    #[test]
    fn test_create_new_exchange() {
        let mut exchanges = Exchanges::new();
        let instance_id = exchanges.create_new_exchange();

        assert!(exchanges
            .unwrap_exchange_from_instance(&instance_id)
            .is_ok());
    }

    #[test]
    fn test_mut_unwrap_exchange_from_instance() {
        let mut exchanges = Exchanges::new();
        let instance_id = exchanges.create_new_exchange();

        assert!(exchanges
            .mut_unwrap_exchange_from_instance(&instance_id)
            .is_ok());
    }

    #[test]
    fn test_unwrap_exchange_from_instance_error() {
        let exchanges = Exchanges::new();
        let instance_id = "invalid_id";

        assert!(exchanges
            .unwrap_exchange_from_instance(instance_id)
            .is_err());
    }

    #[test]
    fn test_add_capital() {
        let mut exchanges = Exchanges::new();
        let instance_id = exchanges.create_new_exchange();

        let result = exchanges.add_capital(&instance_id, "BTC", dec!(12.0));
        assert!(result.is_ok());

        let exchange = exchanges
            .unwrap_exchange_from_instance(&instance_id)
            .unwrap();
        let wallets = exchange.get_wallet();
        assert_eq!(wallets.get("BTC"), Some(&dec!(12.0)));
    }

    #[test]
    fn test_tick() {
        let mut exchanges = Exchanges::new();
        let instance_id = exchanges.create_new_exchange();
        let custom_kline_data = vec![BinanceKline::new(
            1626578400000,
            "1.0000000",
            "2.0000000",
            "0.08000000",
            "0.15000000",
            "5000.00000000",
            1626578500000,
            "750.00000000",
            10,
            "2500.00000000",
            "2500.00000000",
            "0.0",
        )];

        let mut price_feed = PriceFeed::new();
        price_feed.add_price_data(custom_kline_data);

        let mut exchange = Exchange::new();
        exchange
            .with_capital(vec![
                ("BTC".to_string(), dec!(1.0)),
                ("USDT".to_string(), dec!(1.0)),
            ])
            .add_price_feed("BTCUSDT".to_string(), price_feed);
        exchanges.exchanges.insert(instance_id.clone(), exchange);

        // Place a limit buy order
        let exchange = exchanges
            .mut_unwrap_exchange_from_instance(&instance_id)
            .unwrap();
        let _ = exchange
            .place_limit_buy_order("BTCUSDT", dec!(1), dec!(1))
            .unwrap();

        let result = exchanges.tick(&instance_id);
        assert!(result.is_ok());

        let exchange = exchanges
            .unwrap_exchange_from_instance(&instance_id)
            .unwrap();
        let wallets = exchange.get_wallet();
        assert_eq!(wallets["BTC"], dec!(2.0));
        assert_eq!(wallets["USDT"], dec!(0.0));
    }
}
