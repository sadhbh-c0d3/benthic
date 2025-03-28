use std::{fmt, rc::Rc};


#[derive(Clone, Copy)]
pub enum Side {
    Bid,
    Ask
}

pub struct Asset
{
    pub symbol: String,
    pub decimals: u8
}

pub struct Market
{
    pub symbol: String,
    pub base_asset: Rc<Asset>,
    pub quote_asset: Rc<Asset>,
    pub tick: u64,
    pub multiplier: u16,
    pub base_decimals: u8,
    pub quote_decimals: u8
}

pub struct LimitOrder
{
    pub side: Side,
    pub price: u64,
    pub quantity: u64,
}

pub struct MarketOrder
{
    pub side: Side,
    pub quantity: u64,
}

pub enum OrderType {
    ImmediateOrCancel(LimitOrder),
    Limit(LimitOrder),
    Market(MarketOrder)
    // TODO: Add OCO and Stop orders
}

pub struct Order
{
    pub market: Rc<Market>,
    pub participant_id: usize,
    pub order_id: usize,
    pub order_data: OrderType
}

pub fn side_name(side: Side) -> &'static str {
    match side {
        Side::Ask => "sell",
        Side::Bid => "buy"
    }
}

pub fn price_fmt(price: u64, decimals: u8) -> String {
    let base: u64 = 10;
    let k = base.pow(decimals as u32);
    let a = price / k;
    let b = price % k;
    format!("{}.{}", a, b)
}

pub fn calculate_value(quantity: u64, price: u64, base_decimals: u8, quote_decimals: u8) -> u64 {
    let decimal_base: u64 = 10;

    // base = a_base * k_base + b_base
    let k_base = decimal_base.pow(base_decimals as u32);
    let a_base = quantity / k_base;
    let b_base = quantity % k_base;

    // quote = a_quote * k_quote + b_quote
    let k_quote = decimal_base.pow(quote_decimals as u32);
    let a_quote = price / k_quote;
    let b_quote = price % k_quote;

    // base * quote = (a_base * k_base + b_base) * (a_quote * k_quote + b_quote)
    // = (a_base * a_quote * k_base * k_quote) + 
    //   (a_base * b_quote * k_base) + 
    //   (a_quote * b_base * k_quote) +
    //   (b_base * b_quote)
    let a = a_base * a_quote; // * (k_base * k_quote)
    let b = a_base * b_quote; // * (k_base)
    let c = a_quote * b_base; // * (k_quote)
    let d = b_base * b_quote; // * 1

    // base * quote / k_base
    // = (a_base * a_quote * k_quote) + 
    //   (a_base * b_quote) + 
    //   (a_quote * b_base) / k_base +
    //   (b_base * b_quote) / k_base
    // = a * k_quote + b + c / k_base + d / k_base
    //
    a * k_quote + b + (c + d) / k_base
}

pub fn quote_price_fmt(price: u64, market: &Market) -> String {
    format!("{}{}", price_fmt(price, market.quote_decimals), market.quote_asset.symbol)
}

pub fn base_quantity_fmt(quantity: u64, market: &Market) -> String {
    format!("{}{}", price_fmt(quantity, market.base_decimals), market.base_asset.symbol)
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.order_data {
            OrderType::Limit(limit) => write!(f, "Limit {} {} @ {}",
                side_name(limit.side),
                base_quantity_fmt(limit.quantity, &self.market),
                quote_price_fmt(limit.price, &self.market)),

            OrderType::ImmediateOrCancel(limit) => write!(f, "IOC {} {} @ {}",
                side_name(limit.side),
                base_quantity_fmt(limit.quantity, &self.market),
                quote_price_fmt(limit.price, &self.market)),

            OrderType::Market(market_order) => write!(f, "Market {} {}",
                side_name(market_order.side),
                base_quantity_fmt(market_order.quantity, &self.market))
        }
    }
}


#[test]
fn test_calculate_value() {
    let quantity = 150;
    let price = 200;
    let base_decimals = 1;
    let quote_decimals = 2;
    let value = calculate_value(quantity, price, base_decimals, quote_decimals);
    println!("Calculated {} x {} = {} ({})", 
        price_fmt(quantity, base_decimals), 
        price_fmt(price, quote_decimals), 
        price_fmt(value, 2),
        value);
    assert_eq!(value, 3000);
}
