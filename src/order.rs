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


