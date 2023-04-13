use rust_decimal::prelude::Decimal;
use rust_decimal_macros::dec;
use trade_sim::exchange::exchange::Exchange;
use trade_sim::exchange::price_feed::{BinanceKline, PriceFeed};

#[test]
fn test_create_and_place_exchange_order() {
    let custom_kline_data = vec![
        BinanceKline::new(
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
        ),
        BinanceKline::new(
            1626578400000,
            "0.1500000",
            "0.2000000",
            "0.04000000",
            "0.3000000",
            "5000.00000000",
            1626578500000,
            "750.00000000",
            10,
            "2500.00000000",
            "2500.00000000",
            "0.0",
        ),
    ];

    let mut price_feed = PriceFeed::new();
    //custom_kline_data.reverse();
    price_feed.add_price_data(custom_kline_data);
    let mut exchange = Exchange::new()
        .with_capital(vec![
            ("BTC".to_string(), dec!(1.0)),
            ("USDT".to_string(), dec!(1.0)),
        ])
        .add_price_feed("BTCUSDT".to_string(), price_feed);

    // Place a limit sell order
    let _ = exchange
        .place_limit_sell_order("BTCUSDT", dec!(1), dec!(1))
        .unwrap();

    // Place a limit buy order
    let _ = exchange
        .place_limit_buy_order("BTCUSDT", dec!(0.05), dec!(1))
        .unwrap();

    // Call the tick() function
    let result = exchange.tick();
    assert!(result.is_ok());

    let wallets = exchange.get_wallet();
    assert_eq!(wallets["USDT"], dec!(2.0));
    assert_eq!(wallets["BTC"], dec!(0.0));

    // Call the tick() function
    let result = exchange.tick();
    assert!(result.is_ok());

    let wallets = exchange.get_wallet();
    assert_eq!(wallets["USDT"], dec!(1.95));
    assert_eq!(wallets["BTC"], dec!(1.0));
}
