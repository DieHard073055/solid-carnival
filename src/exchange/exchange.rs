use std::collections::HashMap;
use thiserror::Error;

use crate::exchange::order::{Order, OrderDirection, OrderType};
use crate::exchange::price_feed::{BinanceKline, PriceFeed};
use crate::exchange::transaction::Transaction;
use crate::exchange::wallet::Wallet;
use rust_decimal::prelude::Decimal;
use rust_decimal_macros::dec;
use std::fmt::Debug;
use uuid::Uuid;
/*
binance mock exchange ?

new:                create a new instance of the exchange, with some initial capital.
with_price_feed:    This is how you will connect to binance or your custom prices.
place_order :       will place an order which will immediately fill or eventually get filled.
tick:               will pull all the price from binance, to check if any of the orders were filled since the last
                    checked point.
get_wallet:         take a look at the portfolios performance


 */

pub struct Exchange {
    active_orders: HashMap<String, Vec<Order>>,
    wallet: Wallet,
    price_feeds: HashMap<String, PriceFeed>,
    instance_id: String,
}

#[derive(Debug, Clone, Error)]
pub enum ExchangeError {
    #[error("Failed to obtain asset pair")]
    FailedToObtainAssetPair,
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Failed to place order")]
    FailedToPlaceOrder,
    #[error("Failed to obtain a valid price")]
    InvalidPrice,
    #[error("No KlineData")]
    NoKlineDataAvailable,
    #[error("Unable to obtain price from order")]
    NoOrderPriceAvailable,
    #[error("Unable to pull price feed")]
    NoPriceFeed,
}

impl Exchange {
    pub fn new() -> Self {
        Exchange {
            active_orders: HashMap::new(),
            wallet: Wallet::new(),
            price_feeds: HashMap::new(),
            instance_id: Uuid::new_v4().hyphenated().to_string(),
        }
    }
    pub fn with_capital(&mut self, funding: Vec<(String, Decimal)>) -> Self {
        for (symbol, qty) in funding.iter() {
            self.wallet.add(&Transaction::new(
                0i64,
                symbol.clone(),
                dec!(0),
                qty.clone(),
            ));
        }
        self
    }
    pub fn with_price_feed(
        &mut self,
        symbol: String,
        interval: String,
        limit: i32,
    ) -> Result<Self, ExchangeError> {
        self.price_feeds
            .entry(symbol.clone())
            .or_insert(PriceFeed::new())
            .initialize_price_feed(symbol, interval, limit)
            .map_err(|_| ExchangeError::NoPriceFeed)?;
        Ok(self)
    }
    pub fn add_price_feed(mut self, symbol: String, price_feed: PriceFeed) -> Self {
        self.price_feeds.insert(symbol, price_feed);
        self
    }
    pub fn price_feed_next(&mut self, symbol: &str) -> Option<BinanceKline> {
        if let Some(price_feed) = self.price_feeds.get_mut(symbol) {
            return price_feed.next();
        }
        None
    }
    pub fn get_wallet(&self) -> &HashMap<String, Decimal> {
        &self.wallet.get_wallets()
    }
    pub fn get_transactions(&self) -> &Vec<Transaction> {
        &self.wallet.get_transactions()
    }
    pub fn get_orders(&self) -> &HashMap<String, Vec<Order>> {
        &self.active_orders
    }
    pub fn get_instance_id(&self) -> &str {
        self.instance_id.as_str()
    }
    pub fn place_order(
        &mut self,
        pair: &str,
        optional_price: Option<Decimal>,
        qty: Decimal,
        direction: OrderDirection,
        order_type: OrderType,
    ) -> Result<Order, ExchangeError> {
        // Get the base asset and the quote asset
        let (base, quote) = Exchange::get_asset_pair(pair)?;
        // Check if the wallet has the required funds
        if order_type == OrderType::Market {
            unimplemented!("Stop letting the exchange take your money!");
        }
        if let Some(price) = optional_price {
            match direction {
                OrderDirection::Buy => {
                    if let None = self.wallet.has_funds_for_order(quote, price * qty) {
                        return Err(ExchangeError::InsufficientFunds);
                    }
                }
                OrderDirection::Sell => {
                    if let None = self.wallet.has_funds_for_order(base, qty) {
                        return Err(ExchangeError::InsufficientFunds);
                    }
                }
            }
        }

        // Create the order and add to the orders hashmap
        self.active_orders.entry(pair.to_string()).or_insert(vec![]);

        let new_order = Order::new_order(pair, optional_price, qty, direction, order_type);
        self.active_orders
            .entry(pair.to_string())
            .or_insert_with(Vec::new)
            .push(new_order.clone());
        Ok(new_order)
    }
    pub fn place_limit_buy_order(
        &mut self,
        pair: &str,
        price: Decimal,
        qty: Decimal,
    ) -> Result<Order, ExchangeError> {
        self.place_order(
            pair,
            Some(price),
            qty,
            OrderDirection::Buy,
            OrderType::Limit,
        )
    }
    pub fn place_limit_sell_order(
        &mut self,
        pair: &str,
        price: Decimal,
        qty: Decimal,
    ) -> Result<Order, ExchangeError> {
        self.place_order(
            pair,
            Some(price),
            qty,
            OrderDirection::Sell,
            OrderType::Limit,
        )
    }
    pub fn place_market_buy_order(
        &mut self,
        pair: &str,
        price: Decimal,
        qty: Decimal,
    ) -> Result<Order, ExchangeError> {
        self.place_order(
            pair,
            Some(price),
            qty,
            OrderDirection::Buy,
            OrderType::Market,
        )
    }
    pub fn place_market_sell_order(
        &mut self,
        pair: &str,
        price: Decimal,
        qty: Decimal,
    ) -> Result<Order, ExchangeError> {
        self.place_order(
            pair,
            Some(price),
            qty,
            OrderDirection::Sell,
            OrderType::Market,
        )
    }

    pub fn tick(&mut self) -> Result<(), ExchangeError> {
        let mut transactions_to_be_added: Vec<Transaction> = vec![];
        let active_orders = self.active_orders.clone();
        for symbol in self.price_feeds.clone().keys() {
            if let Some(kline_data) = self.price_feed_next(symbol.as_str()) {
                let mut executed_orders: Vec<u64> = vec![];
                let (timestamp, _, high, low, _) = kline_data.get_ohlc();
                for order in &active_orders[symbol] {
                    let is_executed = Self::tick_handle_order(
                        &mut transactions_to_be_added,
                        &symbol,
                        timestamp,
                        high,
                        low,
                        order,
                    )?;
                    if is_executed {
                        executed_orders.push(order.id);
                    }
                }

                self.active_orders
                    .get_mut(symbol)
                    .unwrap()
                    .retain(|order| !executed_orders.contains(&order.id));
            }
        }

        for tx in transactions_to_be_added {
            self.wallet.add(&tx);
        }
        Ok(())
    }

    fn tick_handle_order(
        mut transactions_to_be_added: &mut Vec<Transaction>,
        symbol: &String,
        timestamp: i64,
        high: &str,
        low: &str,
        order: &Order,
    ) -> Result<bool, ExchangeError> {
        let order_price = order.price.ok_or(ExchangeError::NoOrderPriceAvailable)?;
        let (base, quote) = Exchange::get_asset_pair(&symbol)?;
        match order.direction {
            OrderDirection::Buy => Self::tick_handle_buy(
                &mut transactions_to_be_added,
                timestamp,
                low,
                order,
                order_price,
                base,
                quote,
            ),
            OrderDirection::Sell => Self::tick_handle_sell(
                &mut transactions_to_be_added,
                timestamp,
                high,
                order,
                order_price,
                base,
                quote,
            ),
        }
    }
    fn create_transaction_and_add_to_list(
        ts: i64,
        symbol: String,
        order_price: Decimal,
        qty: Decimal,
        transactions_to_be_added: &mut Vec<Transaction>,
    ) {
        transactions_to_be_added.push(Transaction::new(ts, symbol, order_price, qty));
    }

    fn tick_handle_sell(
        transactions_to_be_added: &mut Vec<Transaction>,
        timestamp: i64,
        high_price_str: &str,
        order: &Order,
        order_price: Decimal,
        base: &str,
        quote: &str,
    ) -> Result<bool, ExchangeError> {
        let decimal_high =
            Decimal::from_str_exact(high_price_str).map_err(|_| ExchangeError::InvalidPrice)?;
        if decimal_high > order_price {
            Self::create_transaction_and_add_to_list(
                timestamp,
                base.to_string(),
                order_price,
                order.qty * dec!(-1),
                transactions_to_be_added,
            );
            Self::create_transaction_and_add_to_list(
                timestamp,
                quote.to_string(),
                order_price,
                order.qty * order_price,
                transactions_to_be_added,
            );
            return Ok(true);
        }
        Ok(false)
    }

    fn tick_handle_buy(
        transactions_to_be_added: &mut Vec<Transaction>,
        timestamp: i64,
        low_price_str: &str,
        order: &Order,
        order_price: Decimal,
        base: &str,
        quote: &str,
    ) -> Result<bool, ExchangeError> {
        let decimal_low =
            Decimal::from_str_exact(low_price_str).map_err(|_| ExchangeError::InvalidPrice)?;

        if decimal_low < order_price {
            Self::create_transaction_and_add_to_list(
                timestamp,
                base.to_string(),
                order_price,
                order.qty,
                transactions_to_be_added,
            );
            Self::create_transaction_and_add_to_list(
                timestamp,
                quote.to_string(),
                order_price,
                (order.qty * order_price) * dec!(-1),
                transactions_to_be_added,
            );
            return Ok(true);
        }
        Ok(false)
    }

    pub fn get_asset_pair(pair: &str) -> Result<(&str, &str), ExchangeError> {
        const QUOTE_LIST: [&str; 32] = [
            "AUD", "BIDR", "BKRW", "BNB", "BRL", "BTC", "BUSD", "BVND", "DAI", "DOGE", "DOT",
            "ETH", "EUR", "GBP", "IDRT", "NGN", "PAX", "PLN", "RON", "RUB", "TRX", "TRY", "TUSD",
            "UAH", "USDC", "USDP", "USDS", "USDT", "UST", "VAI", "XRP", "ZAR",
        ];
        for quote in QUOTE_LIST {
            if pair.ends_with(quote) {
                let base = pair
                    .strip_suffix(quote)
                    .ok_or(ExchangeError::FailedToObtainAssetPair)?;
                return Ok((base, quote));
            }
        }
        return Err(ExchangeError::FailedToObtainAssetPair);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::exchange::order::OrderStatus;
    use crate::exchange::price_feed::BinanceKline;

    #[test]
    fn test_initialize_exchange_with_capital() {
        let ex = Exchange::new().with_capital(vec![(String::from("BTC"), dec!(12.0))]);
        let wallets = ex.get_wallet();
        assert_eq!(wallets.get("BTC"), Some(&dec!(12.0)));

        let ex = Exchange::new().with_capital(vec![(String::from("USDC"), dec!(12_000))]);
        let wallets = ex.get_wallet();
        assert_eq!(wallets.get("USDC"), Some(&dec!(12_000)));

        let ex = Exchange::new().with_capital(vec![
            (String::from("BTC"), dec!(3)),
            (String::from("ETH"), dec!(40)),
            (String::from("USDC"), dec!(3_000)),
        ]);
        let wallets = ex.get_wallet();
        assert_eq!(wallets.get("BTC"), Some(&dec!(3)));
        assert_eq!(wallets.get("ETH"), Some(&dec!(40)));
        assert_eq!(wallets.get("USDC"), Some(&dec!(3_000)));
    }

    #[test]
    fn test_place_limit_buy_order() {
        let mut ex = Exchange::new().with_capital(vec![(String::from("BTC"), dec!(12.0))]);
        let wallets = ex.get_wallet();
        assert_eq!(wallets.get("BTC"), Some(&dec!(12.0)));

        let place_order_pair = "ETHBTC";
        let place_order_price = dec!(0.0093);
        let place_order_qty = dec!(1);
        let order = ex
            .place_limit_buy_order(place_order_pair, place_order_price, place_order_qty)
            .unwrap();
        let order_map = ex.get_orders();
        if let Some((_, orders)) = order_map.iter().next() {
            if let Some(order) = orders.get(0) {
                assert_eq!(order.order_type, OrderType::Limit);
                assert_eq!(order.direction, OrderDirection::Buy);
                assert_eq!(order.status, OrderStatus::Pending);
                assert_eq!(order.pair, String::from(place_order_pair));
                assert_eq!(order.price, Some(place_order_price));
                assert_eq!(order.qty, place_order_qty);
            }
        }
    }

    #[test]
    fn test_extract_quote_and_base() {
        let result = Exchange::get_asset_pair("BTCUSDT").unwrap();
        assert_eq!(result, ("BTC", "USDT"));
        let result = Exchange::get_asset_pair("ETHBTC").unwrap();
        assert_eq!(result, ("ETH", "BTC"));
        let result = Exchange::get_asset_pair("LINKBNB").unwrap();
        assert_eq!(result, ("LINK", "BNB"));
        let result = Exchange::get_asset_pair("SANDETH").unwrap();
        assert_eq!(result, ("SAND", "ETH"));
    }

    #[test]
    fn test_tick_with_limit_buy() {
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
        let mut exchange = Exchange::new()
            .with_capital(vec![
                ("BTC".to_string(), dec!(1.0)),
                ("USDT".to_string(), dec!(1.0)),
            ])
            .add_price_feed("BTCUSDT".to_string(), price_feed);

        // Place a limit buy order
        let _ = exchange
            .place_limit_buy_order("BTCUSDT", dec!(1), dec!(1))
            .unwrap();

        // Call the tick() function
        let result = exchange.tick();
        assert!(result.is_ok());

        let wallets = exchange.get_wallet();
        assert_eq!(wallets["BTC"], dec!(2.0));
        assert_eq!(wallets["USDT"], dec!(0.0));
    }
    #[test]
    fn test_tick_with_limit_sell() {
        let custom_kline_data = vec![BinanceKline::new(
            1626578400000,
            "2.90000000",
            "3.0000000",
            "2.08000000",
            "2.815000000",
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
        let mut exchange = Exchange::new()
            .with_capital(vec![
                ("BTC".to_string(), dec!(1.0)),
                ("USDT".to_string(), dec!(1.0)),
            ])
            .add_price_feed("BTCUSDT".to_string(), price_feed);

        // Place a limit buy order
        let _ = exchange
            .place_limit_sell_order("BTCUSDT", dec!(2), dec!(1))
            .unwrap();

        // Call the tick() function
        let result = exchange.tick();
        assert!(result.is_ok());

        let wallets = exchange.get_wallet();
        assert_eq!(wallets["BTC"], dec!(0.0));
        assert_eq!(wallets["USDT"], dec!(3.0));
    }
}
