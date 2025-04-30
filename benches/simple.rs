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
use rand::{rngs::SmallRng, Rng, SeedableRng};

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

fn benchmark_simple(c: &mut Criterion) {
    let asset_usdt = Rc::new(Asset {
        symbol: "USDT".into(),
        decimals: 2,
    });

    let asset_btc = Rc::new(Asset {
        symbol: "BTC".into(),
        decimals: 7,
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

    let order_books = Rc::new(OrderBooks::new(&[Rc::new(RefCell::new(OrderBook::new(
        market_btc_usdt.clone(),
    )))]));

    let mut order_manager = OrderManager::new(order_books);
    let mut margin_manager = MarginManager::new(MarginLotEventHandlerNull);

    let mut rng = SmallRng::seed_from_u64(123456999);

    (0..4).for_each(|n| {
        margin_manager
            .add_account(n)
            .borrow_mut()
            .add_asset_account(&asset_btc)
            .add_asset_account(&asset_usdt)
            .transfer(
                Rc::new(Order {
                    market: market_btc_usdt.clone(),
                    participant_id: n,
                    order_id: n,
                    order_data: OrderType::Deposit(rng.random_range(1_00000..100_00000)),
                }),
                rng.random_range(400000..10000000),
            )
            .expect("Failed to create account");
    });

    let user1 = 0;
    let user2 = 1;
    let user3 = 2;
    let user4 = 3;

    let orders = [
        Rc::new(Order {
            market: market_btc_usdt.clone(),
            order_id: 0,
            participant_id: user4,
            order_data: OrderType::Limit(LimitOrder {
                side: Side::Bid,
                price: rng.random_range(500_0000..940_0000),
                quantity: 45,
            }),
        }),
        Rc::new(Order {
            market: market_btc_usdt.clone(),
            order_id: 1,
            participant_id: user3,
            order_data: OrderType::Limit(LimitOrder {
                side: Side::Bid,
                price: rng.random_range(950_0000..1050_0000),
                quantity: 15,
            }),
        }),
        Rc::new(Order {
            market: market_btc_usdt.clone(),
            order_id: 2,
            participant_id: user1,
            order_data: OrderType::Limit(LimitOrder {
                side: Side::Bid,
                price: rng.random_range(1200_0000..1500_0000),
                quantity: 20,
            }),
        }),
        Rc::new(Order {
            market: market_btc_usdt.clone(),
            order_id: 3,
            participant_id: user2,
            order_data: OrderType::Limit(LimitOrder {
                side: Side::Ask,
                price: rng.random_range(1100_0000..1300_0000),
                quantity: 10,
            }),
        }),
        Rc::new(Order {
            market: market_btc_usdt.clone(),
            order_id: 4,
            participant_id: user3,
            order_data: OrderType::Limit(LimitOrder {
                side: Side::Ask,
                price: rng.random_range(1100_0000..1400_0000),
                quantity: 15,
            }),
        }),
        Rc::new(Order {
            market: market_btc_usdt.clone(),
            order_id: 5,
            participant_id: user1,
            order_data: OrderType::Limit(LimitOrder {
                side: Side::Bid,
                price: rng.random_range(1250_0000..1800_0000),
                quantity: 5,
            }),
        }),
        Rc::new(Order {
            market: market_btc_usdt.clone(),
            order_id: 1,
            participant_id: user3,
            order_data: OrderType::Cancel,
        }),
        Rc::new(Order {
            market: market_btc_usdt.clone(),
            order_id: 6,
            participant_id: user1,
            order_data: OrderType::Limit(LimitOrder {
                side: Side::Ask,
                price: rng.random_range(500_0000..940_0000),
                quantity: 5,
            }),
        }),
        Rc::new(Order {
            market: market_btc_usdt.clone(),
            order_id: 7,
            participant_id: user2,
            order_data: OrderType::Limit(LimitOrder {
                side: Side::Ask,
                price: rng.random_range(1250_0000..1900_0000),
                quantity: 100,
            }),
        }),
        Rc::new(Order {
            market: market_btc_usdt.clone(),
            order_id: 8,
            participant_id: user3,
            order_data: OrderType::Limit(LimitOrder {
                side: Side::Bid,
                price: rng.random_range(950_0000..1100_0000),
                quantity: 15,
            }),
        }),
        Rc::new(Order {
            market: market_btc_usdt.clone(),
            order_id: 9,
            participant_id: user4,
            order_data: OrderType::Limit(LimitOrder {
                side: Side::Ask,
                price: rng.random_range(1300_0000..1500_0000),
                quantity: 30,
            }),
        }),
        Rc::new(Order {
            market: market_btc_usdt.clone(),
            order_id: 7,
            participant_id: user2,
            order_data: OrderType::Cancel,
        }),
        Rc::new(Order {
            market: market_btc_usdt.clone(),
            order_id: 9,
            participant_id: user4,
            order_data: OrderType::Cancel,
        }),
    ];

    let execution_policy = BenchExecutions::new(margin_manager);
    let market_data_policy = MarketDataNull {};

    let execute_orders = |order_manager: &mut OrderManager, orders: &[Rc<Order>]| {
        for order in orders {
            let _ =
                order_manager.place_order(order.clone(), &execution_policy, &market_data_policy);
        }
    };
    
    let time_started = Utc::now();

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

criterion_group!(benches, benchmark_simple);
criterion_main!(benches);
