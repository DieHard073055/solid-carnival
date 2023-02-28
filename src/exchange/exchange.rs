use std::collections::HashMap;
use std::error;

use crate::exchange::transaction::Transaction;
use crate::exchange::wallet::Wallet;
use chrono::Utc;
use rust_decimal::prelude::Decimal;
use rust_decimal::Error;
use rust_decimal_macros::dec;
use std::fmt::{Debug, Display, Formatter};
/*
binance mock exchange ?

new:                create a new instance of the exchange, with some initial capital.
with_price_feed:    This is how you will connect to binance or your custom prices.
place_order :       will place an order which will immediately fill or eventually get filled.
tick:               will pull all the price from binance, to check if any of the orders were filled since the last
                    checked point.
get_wallet:         take a look at the portfolios performance


 */
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
    ts: i64,
    order_type: OrderType,
    direction: OrderDirection,
    pair: String,
    price: Option<Decimal>,
    qty: Decimal,
    status: OrderStatus,
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

pub struct Exchange {
    active_orders: HashMap<String, Vec<Order>>,
    wallet: Wallet,
}

#[derive(Debug, Clone)]
enum ExchangeError{
    FailedToObtainAssetPair,
    InsufficientFunds,
    FailedToPlaceOrder,
}

impl Display for ExchangeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unable to extract quote asset from the provided pair.")
    }
}

impl error::Error for ExchangeError {}

impl Exchange {
    pub fn new() -> Self {
        Exchange {
            active_orders: HashMap::new(),
            wallet: Wallet::new(),
        }
    }
    pub fn with_capital(mut self, funding: Vec<(String, Decimal)>) -> Self {
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
    pub fn get_wallet(&self) -> &HashMap<String, Decimal> {
        &self.wallet.get_wallets()
    }
    pub fn get_orders(&self) -> &HashMap<String, Vec<Order>>{
        &self.active_orders
    }
    pub fn place_limit_buy_order(&mut self, pair: &str, price: Decimal, qty: Decimal) -> Result<Order, Box<dyn std::error::Error>> {
        // TODO: create an error message for failing to place a buy order
        // Get the base asset and the quote asset
        let (base, quote) = Exchange::get_asset_pair(pair)?;
        // Check if the wallet has the required funds
        if let None = self.wallet.has_funds_for_order(quote, price * qty){
            return Err(ExchangeError::InsufficientFunds.into());
        }

        // Create the order and add to the orders vector? hashmap?
        self.active_orders.entry(pair.to_string()).or_insert(vec![]);

        if let Some(mut order_list) = self.active_orders.get_mut(pair){
            let new_order = Order::new_limit_buy(pair, price, qty);
            order_list.push(
                new_order.clone()
            );
            return Ok(new_order)
        };

        Err(ExchangeError::FailedToPlaceOrder.into())

    }

    pub fn tick(&mut self){
        // TODO: create an error for all the possible failures
        // Use the price feed object to get the price for all the coins we are listening for
        // Check if any of the orders have been hit
        // Update the wallet accounts if the orders have been hit.
    }

    pub fn get_asset_pair(pair: &str) -> Result<(&str, &str), Box<dyn error::Error>> {
        let quote_list = [
            "AUD", "BIDR", "BKRW", "BNB", "BRL", "BTC", "BUSD", "BVND", "DAI", "DOGE", "DOT",
            "ETH", "EUR", "GBP", "IDRT", "NGN", "PAX", "PLN", "RON", "RUB", "TRX", "TRY", "TUSD",
            "UAH", "USDC", "USDP", "USDS", "USDT", "UST", "VAI", "XRP", "ZAR",
        ];
        for quote in quote_list {
            if pair.ends_with(quote) {
                let base = match pair.split(quote).next() {
                    None => return Err(ExchangeError::FailedToObtainAssetPair.into()),
                    Some(b) => b,
                };
                return Ok((base, quote));
            }
        }
        return Err(ExchangeError::FailedToObtainAssetPair.into());
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
        let order = ex.place_limit_buy_order(place_order_pair, place_order_price, place_order_qty).unwrap();
        let order_map = ex.get_orders();
        if let Some((_, orders)) = order_map.iter().next(){
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
}
