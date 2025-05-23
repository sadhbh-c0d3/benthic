use std::{fmt, rc::Rc};

#[derive(Clone, Copy)]
pub enum Side {
    Bid,
    Ask,
}

impl Side {
    pub fn opposite(&self) -> Self {
        match self {
            Self::Bid => Self::Ask,
            Self::Ask => Self::Bid,
        }
    }
}

pub struct Asset {
    pub symbol: String,
    pub decimals: u8,
}

pub struct Market {
    pub symbol: String,
    pub base_asset: Rc<Asset>,
    pub quote_asset: Rc<Asset>,
    pub tick: u64,
    pub multiplier: u16,
    pub base_decimals: u8,
    pub quote_decimals: u8,
}

pub struct LimitOrder {
    pub side: Side,
    pub price: u64,
    pub quantity: u64,
}

pub struct MarketOrder {
    pub side: Side,
    pub quantity: u64,
}

pub enum OrderType {
    Deposit(u64),
    Withdraw(u64),
    ImmediateOrCancel(LimitOrder),
    Limit(LimitOrder),
    Market(MarketOrder), // TODO: Add OCO and Stop orders
}

pub struct Order {
    pub market: Rc<Market>,
    pub participant_id: usize,
    pub order_id: usize,
    pub order_data: OrderType,
}

impl Order {
    pub fn get_quantity_and_value(&self, quantity: u64, price: u64) -> Option<(u64, u64)> {
        let order_value = calculate_value(
            quantity,
            price,
            self.market.base_decimals,
            self.market.quote_decimals,
        )?;

        let order_quantity_changed = change_decimals(
            quantity,
            self.market.base_decimals,
            self.market.base_asset.decimals,
        )?;

        let order_value_changed = change_decimals(
            order_value,
            self.market.quote_decimals,
            self.market.quote_asset.decimals,
        )?;

        Some((order_quantity_changed, order_value_changed))
    }
}

pub fn side_name(side: Side) -> &'static str {
    match side {
        Side::Ask => "sell",
        Side::Bid => "buy",
    }
}

pub fn transaction_direction(side: Side) -> &'static str {
    match side {
        Side::Ask => "deliver",
        Side::Bid => "receive",
    }
}

pub fn lot_side(side: Side) -> &'static str {
    match side {
        Side::Ask => "Short",
        Side::Bid => "Long",
    }
}

pub fn price_fmt(price: u64, decimals: u8) -> String {
    let base: u64 = 10;
    let k = base.pow(decimals as u32);
    let a = price / k;
    let b = price % k;
    format!("{: >}.{: <}", a, b)
}

pub fn change_decimals(quantity: u64, from_decimals: u8, to_decimals: u8) -> Option<u64> {
    let decimal_base: u64 = 10;
    if from_decimals < to_decimals {
        quantity
            .checked_mul(decimal_base.checked_pow((to_decimals as u32) - (from_decimals as u32))?)
    } else {
        quantity
            .checked_div(decimal_base.checked_pow((from_decimals as u32) - (to_decimals as u32))?)
    }
}

pub fn calculate_value(
    quantity: u64,
    price: u64,
    base_decimals: u8,
    quote_decimals: u8,
) -> Option<u64> {
    let decimal_base: u64 = 10;

    // base = a_base * k_base + b_base
    let k_base = decimal_base.checked_pow(base_decimals as u32)?;
    let a_base = quantity / k_base;
    let b_base = quantity % k_base;

    // quote = a_quote * k_quote + b_quote
    let k_quote = decimal_base.checked_pow(quote_decimals as u32)?;
    let a_quote = price / k_quote;
    let b_quote = price % k_quote;

    // base * quote = (a_base * k_base + b_base) * (a_quote * k_quote + b_quote)
    // = (a_base * a_quote * k_base * k_quote) +
    //   (a_base * b_quote * k_base) +
    //   (a_quote * b_base * k_quote) +
    //   (b_base * b_quote)
    let a = a_base.checked_mul(a_quote)?; // * (k_base * k_quote)
    let b = a_base.checked_mul(b_quote)?; // * (k_base)
    let c = a_quote.checked_mul(b_base)?; // * (k_quote)
    let d = b_base.checked_mul(b_quote)?; // * 1

    // base * quote / k_base
    // = (a_base * a_quote * k_quote) +
    //   (a_base * b_quote) +
    //   (a_quote * b_base) / k_base +
    //   (b_base * b_quote) / k_base
    // = a * k_quote + b + c / k_base + d / k_base
    //
    a.checked_mul(k_quote)?.checked_add(b)?.checked_add(
        c.checked_mul(k_quote)?
            .checked_add(d)?
            .checked_div(k_base)?,
    )
}

pub fn quote_price_fmt(price: u64, market: &Market) -> String {
    format!(
        "{}{}",
        price_fmt(price, market.quote_decimals),
        market.quote_asset.symbol
    )
}

pub fn base_quantity_fmt(quantity: u64, market: &Market) -> String {
    format!(
        "{}{}",
        price_fmt(quantity, market.base_decimals),
        market.base_asset.symbol
    )
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.order_data {
            OrderType::Limit(limit) => write!(
                f,
                "Limit {} {} @ {}",
                side_name(limit.side),
                base_quantity_fmt(limit.quantity, &self.market),
                quote_price_fmt(limit.price, &self.market)
            ),

            OrderType::ImmediateOrCancel(limit) => write!(
                f,
                "IOC {} {} @ {}",
                side_name(limit.side),
                base_quantity_fmt(limit.quantity, &self.market),
                quote_price_fmt(limit.price, &self.market)
            ),

            OrderType::Market(market_order) => write!(
                f,
                "Market {} {}",
                side_name(market_order.side),
                base_quantity_fmt(market_order.quantity, &self.market)
            ),
            OrderType::Deposit(quantity) => {
                write!(f, "Deposit {}", base_quantity_fmt(*quantity, &self.market))
            }
            OrderType::Withdraw(quantity) => {
                write!(f, "Withdraw {}", base_quantity_fmt(*quantity, &self.market))
            }
        }
    }
}

#[test]
fn test_calculate_value() {
    let quantity = 150;
    let price = 200;
    let base_decimals = 1;
    let quote_decimals = 2;
    let value = calculate_value(quantity, price, base_decimals, quote_decimals).unwrap();
    println!(
        "Calculated {} x {} = {} ({})",
        price_fmt(quantity, base_decimals),
        price_fmt(price, quote_decimals),
        price_fmt(value, 2),
        value
    );
    assert_eq!(value, 3000);

    let base_asset_decimals = 2;
    let quote_asset_decimals = 1;
    let quantity_changed = change_decimals(quantity, base_decimals, base_asset_decimals).unwrap();
    println!(
        "Changed decimals {} => {}",
        price_fmt(quantity, base_decimals),
        price_fmt(quantity_changed, base_asset_decimals)
    );
    assert_eq!(quantity_changed, 1500);

    let value_changed = change_decimals(value, quote_decimals, quote_asset_decimals).unwrap();
    println!(
        "Changed decimals {} => {}",
        price_fmt(value, quote_decimals),
        price_fmt(value_changed, quote_asset_decimals)
    );
    assert_eq!(value_changed, 300);
}

#[test]
fn test_calculate_value_2() {
    let quantity = 50000;
    let price = 125000;
    let base_decimals = 5;
    let quote_decimals = 4;
    let value = calculate_value(quantity, price, base_decimals, quote_decimals).unwrap();
    println!(
        "Calculated {} x {} = {} ({})",
        price_fmt(quantity, base_decimals),
        price_fmt(price, quote_decimals),
        price_fmt(value, quote_decimals),
        value
    );
    assert_eq!(value, 62500);

    let base_asset_decimals = 7;
    let quote_asset_decimals = 6;
    let quantity_changed = change_decimals(quantity, base_decimals, base_asset_decimals).unwrap();
    println!(
        "Changed decimals {} => {}",
        price_fmt(quantity, base_decimals),
        price_fmt(quantity_changed, base_asset_decimals)
    );
    assert_eq!(quantity_changed, 5000000);

    let value_changed = change_decimals(value, quote_decimals, quote_asset_decimals).unwrap();
    println!(
        "Changed decimals {} => {}",
        price_fmt(value, quote_decimals),
        price_fmt(value_changed, quote_asset_decimals)
    );
    assert_eq!(value_changed, 6250000);
}
