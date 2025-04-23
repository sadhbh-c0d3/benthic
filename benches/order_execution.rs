use std::{cell::RefCell, rc::Rc};

use benthic::{
    execution_policy::ExecutionPolicy,
    margin::{MarginLotEventHandlerNull, MarginManager},
    market_data_policy::MarketDataNull,
    order::{Asset, LimitOrder, Market, Order, OrderType, Side},
    order_book::OrderBook,
    order_manager::{OrderBooks, OrderManager},
};
use chrono::Utc;
use criterion::{criterion_group, criterion_main, Criterion};
use itertools::Itertools;
use rand::{rngs::SmallRng, Rng, SeedableRng};

const NUM_TRADERS: usize = 1_000;
const NUM_ORDERS: usize = 500_000;
const BENCHMARK_VERSION: &str = "Static Lots Handler (VecDeque)";

struct BenchExecutions<T>
where
    T: ExecutionPolicy,
{
    policy: T,
    pub placed_order_count: RefCell<usize>,
    pub executed_order_count: RefCell<usize>,
}

impl<T> BenchExecutions<T>
where
    T: ExecutionPolicy,
{
    pub fn new(policy: T) -> Self {
        Self {
            policy,
            placed_order_count: RefCell::new(0),
            executed_order_count: RefCell::new(0),
        }
    }
}

impl<T> ExecutionPolicy for BenchExecutions<T>
where
    T: ExecutionPolicy,
{
    fn place_order(
        &self,
        order_quantity: &mut benthic::order_book::OrderQuantity,
    ) -> Result<(), Box<dyn std::error::Error>> {
        *self.placed_order_count.borrow_mut() += 1;
        self.policy.place_order(order_quantity)
    }
    fn cancel_order(
        &self,
        order_quantity: &mut benthic::order_book::OrderQuantity,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.policy.cancel_order(order_quantity)
    }
    fn execute_orders(
        &self,
        executed_quantity: &mut u64,
        aggressor_order: &mut benthic::order_book::OrderQuantity,
        book_order: &mut benthic::order_book::OrderQuantity,
    ) -> Result<(), Box<dyn std::error::Error>> {
        *self.executed_order_count.borrow_mut() += 1;
        self.policy
            .execute_orders(executed_quantity, aggressor_order, book_order)
    }
}

fn benchmark_order_placement(c: &mut Criterion) {
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

    let mut order_manager = OrderManager::new(order_books);

    let mut margin_manager = MarginManager::new(MarginLotEventHandlerNull);

    let mut rng = SmallRng::seed_from_u64(123456999);

    (0..NUM_TRADERS).for_each(|n| {
        margin_manager
            .add_account(n)
            .borrow_mut()
            .add_asset_account(&asset_btc)
            .add_asset_account(&asset_eth)
            .add_asset_account(&asset_usdt)
            .transfer(
                Rc::new(Order {
                    market: if rng.random_bool(0.5) {
                        market_btc_usdt.clone()
                    } else {
                        market_eth_usdt.clone()
                    },
                    participant_id: n,
                    order_id: n,
                    order_data: OrderType::Deposit(rng.random_range(1_00000..100_00000)),
                }),
                rng.random_range(400000..10000000),
            )
            .expect("Failed to create account");
    });

    let orders = (0..NUM_ORDERS)
        .map(|n| {
            Rc::new(Order {
                market: market_btc_eth.clone(),
                order_id: NUM_TRADERS + n,
                participant_id: rng.random_range(0..NUM_TRADERS),
                order_data: OrderType::Limit(LimitOrder {
                    side: if rng.random_bool(0.5) {
                        Side::Bid
                    } else {
                        Side::Ask
                    },
                    price: rng.random_range(10_0000..20_0000),
                    quantity: rng.random_range(1_00000..100_00000),
                }),
            })
        })
        .collect_vec();

    let execution_policy = BenchExecutions::new(margin_manager);
    let market_data_policy = MarketDataNull {};

    let time_started = Utc::now();

    let execute_orders = |order_manager: &mut OrderManager, orders: &[Rc<Order>]| {
        for order in orders {
            let _ =
                order_manager.place_order(order.clone(), &execution_policy, &market_data_policy);
        }
    };

    println!("Config: NUM_TRADERS = {NUM_TRADERS}, NUM_ORDERS = {NUM_ORDERS}, BENCHMARK_VERSION = {BENCHMARK_VERSION}");

    println!(
        "Warm-up: time {}s, orders {}, executions {}",
        (Utc::now() - time_started).num_seconds(),
        execution_policy.placed_order_count.borrow(),
        execution_policy.executed_order_count.borrow(),
    );

    for _ in 0..100 {
        execute_orders(&mut order_manager, &orders);
    }

    println!(
        "Ready: time {}s, orders {}, executions {}",
        (Utc::now() - time_started).num_seconds(),
        execution_policy.placed_order_count.borrow(),
        execution_policy.executed_order_count.borrow(),
    );

    c.bench_function("order_execution_mixed", |b| {
        b.iter(|| {
            execute_orders(&mut order_manager, &orders);
        });
    });

    println!(
        "Finished: time {}s, orders {}, executions {}",
        (Utc::now() - time_started).num_seconds(),
        execution_policy.placed_order_count.borrow(),
        execution_policy.executed_order_count.borrow()
    );
}

criterion_group!(benches, benchmark_order_placement);
criterion_main!(benches);
