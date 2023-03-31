use reqwest::blocking::Response;
use rust_decimal::prelude::Decimal;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Write;

// https://api.binance.com/api/v3/klines?symbol=BTCBUSD&interval=1h&limit=10
const BINANCE_API: &str = "https://api.binance.com/api/v3";
const KLINES: &str = "klines";

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(
    expecting = "expecting [<open_timestamp>, <open>, <high>, <low>, <close>, <volume>, <close_timestamp>, <quote_volume>, <trades>, <bid_volume>, <ask_volume>, <ignore>,] array"
)]
pub struct BinanceKline {
    open_timestamp: i64,
    open: String,
    high: String,
    low: String,
    close: String,
    volume: String,
    close_timestamp: i64,
    quote_volume: String,
    trades: i32,
    bid_volume: String,
    ask_volume: String,
    ignore: String,
}
impl BinanceKline {
    pub fn new(
        open_timestamp: i64,
        open: &str,
        high: &str,
        low: &str,
        close: &str,
        volume: &str,
        close_timestamp: i64,
        quote_volume: &str,
        trades: i32,
        bid_volume: &str,
        ask_volume: &str,
        ignore: &str,
    ) -> Self {
        BinanceKline {
            open_timestamp,
            open: open.to_string(),
            high: high.to_string(),
            low: low.to_string(),
            close: close.to_string(),
            volume: volume.to_string(),
            close_timestamp,
            quote_volume: quote_volume.to_string(),
            trades,
            bid_volume: bid_volume.to_string(),
            ask_volume: ask_volume.to_string(),
            ignore: ignore.to_string(),
        }
    }
    pub fn get_ohlc(&self) -> (i64, &str, &str, &str, &str){
        (self.close_timestamp, self.open.as_str(), self.high.as_str(), self.low.as_str(), self.close.as_str())
    }
}
#[derive(Clone, Debug)]
pub struct PriceFeed {
    cursor: usize,
    price_data: Option<Vec<BinanceKline>>,
}
impl PriceFeed {
    pub fn new() -> Self {
        PriceFeed {
            cursor: 0usize,
            price_data: None,
        }
    }
    pub fn initialize_price_feed(&mut self, symbol: String, interval: String, limit: i32) -> Result<(), Box<dyn Error>>{
        self.price_data = Some(PriceFeed::fetch(symbol, interval, limit)?);
        Ok(())
    }
    pub fn add_price_data(&mut self, klines: Vec<BinanceKline>) {
        self.price_data = Some(klines);
        self.cursor = 0;
    }
    fn save_price_data(
        filename: String,
        price_data: &Vec<BinanceKline>,
    ) -> Result<(), Box<dyn Error>> {
        let mut f = File::create(filename)?;
        let serialized = serde_json::to_string(price_data)?;
        f.write_all(serialized.as_bytes())?;
        Ok(())
    }
    fn fetch(
        symbol: String,
        interval: String,
        limit: i32,
    ) -> Result<Vec<BinanceKline>, Box<dyn Error>> {
        let fetch_filename = format!("data/{:}{:}{:}", symbol, interval, limit);
        let price_data: Vec<BinanceKline>;
        if let Ok(file) = File::open(&fetch_filename) {
            let mut buf_reader = BufReader::new(file);
            let mut contents = String::new();
            buf_reader.read_to_string(&mut contents)?;
            price_data = serde_json::from_str(contents.as_str())?;
        } else {
            let arguments = format!("symbol={:}&interval={:}&limit={:}", symbol, interval, limit);
            let url = format!("{:}/{:}?{:}", BINANCE_API, KLINES, arguments);
            price_data = reqwest::blocking::get(url)?.json()?;
            PriceFeed::save_price_data(fetch_filename, &price_data)?;
        }

        Ok(price_data)
    }
    pub fn next(&mut self) -> Option<BinanceKline> {
        let price_data = self.price_data.as_ref().unwrap();
        if (self.cursor) < price_data.len() {
            let data_out = price_data[self.cursor].clone();
            self.cursor += 1;
            return Some(data_out);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_klines() -> Vec<BinanceKline> {
        vec![
            BinanceKline {
                open_timestamp: 1633064400000,
                open: "55000.00".to_string(),
                high: "55100.00".to_string(),
                low: "54900.00".to_string(),
                close: "55050.00".to_string(),
                volume: "1000.00".to_string(),
                close_timestamp: 1633067999999,
                quote_volume: "55050000.00".to_string(),
                trades: 100,
                bid_volume: "500.00".to_string(),
                ask_volume: "500.00".to_string(),
                ignore: "0".to_string(),
            },
            BinanceKline {
                open_timestamp: 1633068000000,
                open: "55050.00".to_string(),
                high: "55200.00".to_string(),
                low: "54950.00".to_string(),
                close: "55100.00".to_string(),
                volume: "1100.00".to_string(),
                close_timestamp: 1633071599999,
                quote_volume: "60505000.00".to_string(),
                trades: 110,
                bid_volume: "600.00".to_string(),
                ask_volume: "500.00".to_string(),
                ignore: "0".to_string(),
            },
        ]
    }

    #[test]
    fn test_get_ohlc() {
        let kline = &sample_klines()[0];
        let ohlc = kline.get_ohlc();
        assert_eq!(ohlc, (1633067999999, "55000.00", "55100.00", "54900.00", "55050.00"));
    }

    #[test]
    fn test_price_feed_next() {
        let mut price_feed = PriceFeed::new();
        price_feed.add_price_data(sample_klines());

        let kline1 = price_feed.next().unwrap();
        assert_eq!(kline1.get_ohlc(), (1633067999999, "55000.00", "55100.00", "54900.00", "55050.00"));

        let kline2 = price_feed.next().unwrap();
        assert_eq!(kline2.get_ohlc(), (1633071599999, "55050.00", "55200.00", "54950.00", "55100.00"));

        assert!(price_feed.next().is_none());
    }
}
