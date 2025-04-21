use std::{cell::RefCell, rc::Rc};

use itertools::Itertools;

use benthic::{
    margin::MarginManager,
    market_data_policy::MarketDataNull,
    order::{price_fmt, Asset, LimitOrder, Market, Order, OrderType, Side},
    order_book::OrderBook,
    order_manager::{LogExecutions, LogMarketData, OrderBooks, OrderManager},
};

fn main() {
    let asset_usdt = Rc::new(Asset {
        symbol: "USDT".into(),
        decimals: 2,
    });

    let asset_btc = Rc::new(Asset {
        symbol: "BTC".into(),
        decimals: 7,
    });

    let asset_eth = Rc::new(Asset {
        symbol: "ETH".into(),
        decimals: 6,
    });

    let market_btc_usdt = Rc::new(Market {
        symbol: "BTC/USDT".into(),
        base_asset: asset_btc.clone(),
        quote_asset: asset_usdt.clone(),
        tick: 1,
        multiplier: 1,
        quote_decimals: 2,
        base_decimals: 5,
    });

    let market_eth_usdt = Rc::new(Market {
        symbol: "ETH/USDT".into(),
        base_asset: asset_eth.clone(),
        quote_asset: asset_usdt.clone(),
        tick: 1,
        multiplier: 1,
        quote_decimals: 2,
        base_decimals: 5,
    });

    let market_btc_eth = Rc::new(Market {
        symbol: "BTC/ETH".into(),
        base_asset: asset_btc.clone(),
        quote_asset: asset_eth.clone(),
        tick: 1,
        multiplier: 1,
        quote_decimals: 4,
        base_decimals: 5,
    });

    let order_books = Rc::new(OrderBooks::new(&[
        Rc::new(RefCell::new(OrderBook::new(market_btc_usdt.clone()))),
        Rc::new(RefCell::new(OrderBook::new(market_btc_eth.clone()))),
    ]));

    let trader_a = 1001;
    let trader_b = 1002;

    let mut order_manager = OrderManager::new(order_books);
    let mut margin_manager = MarginManager::new();
    println!("Margin  -->  create Account({})", trader_a);
    margin_manager
        .add_account(trader_a)
        .borrow_mut()
        .add_asset_account(&asset_btc)
        .add_asset_account(&asset_eth)
        .add_asset_account(&asset_usdt)
        .transfer(
            Rc::new(Order {
                market: market_btc_usdt.clone(),
                participant_id: trader_a,
                order_id: 101,
                order_data: OrderType::Deposit(200000),
            }),
            5000000,
        )
        .expect("Failed to create account");
    println!("Margin  -->  create Account({})", trader_b);
    margin_manager
        .add_account(trader_b)
        .borrow_mut()
        .add_asset_account(&asset_btc)
        .add_asset_account(&asset_eth)
        .add_asset_account(&asset_usdt)
        .transfer(
            Rc::new(Order {
                market: market_eth_usdt.clone(),
                participant_id: trader_b,
                order_id: 102,
                order_data: OrderType::Deposit(2000000),
            }),
            400000,
        )
        .expect("Failed to create account");
    //let execution_policy = LogExecutions::new(ExecuteAllways{});
    let execution_policy = LogExecutions::new(margin_manager);
    let market_data_policy = LogMarketData::new(MarketDataNull {});

    let orders = [
        Rc::new(Order {
            market: market_btc_usdt.clone(),
            order_id: 1,
            participant_id: trader_a,
            order_data: OrderType::Limit(LimitOrder {
                side: Side::Bid,
                price: 5000000,
                quantity: 100000,
            }),
        }),
        Rc::new(Order {
            market: market_btc_eth.clone(),
            order_id: 2,
            participant_id: trader_a,
            order_data: OrderType::Limit(LimitOrder {
                side: Side::Ask,
                price: 125000,
                quantity: 100000,
            }),
        }),
        Rc::new(Order {
            market: market_btc_eth.clone(),
            order_id: 3,
            participant_id: trader_b,
            order_data: OrderType::Limit(LimitOrder {
                side: Side::Bid,
                price: 125000,
                quantity: 50000,
            }),
        }),
        Rc::new(Order {
            market: market_btc_eth.clone(),
            order_id: 4,
            participant_id: trader_b,
            order_data: OrderType::Limit(LimitOrder {
                side: Side::Bid,
                price: 120000,
                quantity: 100000,
            }),
        }),
        Rc::new(Order {
            market: market_btc_eth.clone(),
            order_id: 5,
            participant_id: trader_b,
            order_data: OrderType::Limit(LimitOrder {
                side: Side::Bid,
                price: 140000,
                quantity: 100000,
            }),
        }),
        Rc::new(Order {
            market: market_btc_eth.clone(),
            order_id: 6,
            participant_id: trader_b,
            order_data: OrderType::Limit(LimitOrder {
                side: Side::Bid,
                price: 150000,
                quantity: 100000,
            }),
        }),
    ];

    let execute_orders = |order_manager: &mut OrderManager, orders: &[Rc<Order>]| {
        for order in orders {
            println!(
                "User --->    Order({}:{}) {}",
                order.participant_id, order.order_id, order
            );
            if let Err(err) =
                order_manager.place_order(order.clone(), &execution_policy, &market_data_policy)
            {
                println!("Error {}", err);
            }
        }
    };

    let print_portfolio = |margin_manager: &MarginManager| {
        println!("");
        for (account_id, account) in margin_manager
            .get_participants()
            .iter()
            .collect_vec()
            .into_iter()
            .sorted_by_key(|x| x.0)
        {
            println!(
                "Account {: >5}   {: >12} {: >10} | {: >10} {: >12}",
                account_id, "(Open)", "Short", "Long", "(Open)",
            );
            println!("----------------------------------------------------------------");

            for (symbol, asset_data) in account
                .borrow()
                .portfolio
                .iter()
                .collect_vec()
                .into_iter()
                .sorted_by_key(|x| x.0)
            {
                let asset_data = asset_data.borrow();
                println!(
                    "\t{: >5}   {: >12} {: >10} | {: >10} {: >12}",
                    symbol,
                    format!(
                        "({})",
                        price_fmt(
                            asset_data.delivered.quantity_open,
                            asset_data.asset.decimals
                        )
                    ),
                    price_fmt(
                        asset_data.delivered.quantity_committed,
                        asset_data.asset.decimals
                    ),
                    price_fmt(
                        asset_data.received.quantity_committed,
                        asset_data.asset.decimals
                    ),
                    format!(
                        "({})",
                        price_fmt(asset_data.received.quantity_open, asset_data.asset.decimals)
                    ),
                );
            }

            println!("");
        }
    };

    print_portfolio(execution_policy.inner());

    execute_orders(&mut order_manager, &orders[..2]);

    print_portfolio(execution_policy.inner());

    execute_orders(&mut order_manager, &orders[2..3]);

    print_portfolio(execution_policy.inner());

    execute_orders(&mut order_manager, &orders[3..5]);

    print_portfolio(execution_policy.inner());

    execute_orders(&mut order_manager, &orders[5..]);

    print_portfolio(execution_policy.inner());
}
