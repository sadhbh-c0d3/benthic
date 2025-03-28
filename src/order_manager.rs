use std::{cell::RefCell, collections::HashMap, error::Error, rc::Rc};

use crate::{execution_policy::{ExecuteAllways, ExecutionPolicy}, margin::MarginManager, order::*, order_book::{OrderBook, OrderQuantity}};

pub trait OrderBookManager {
    fn get_order_book(&self, symbol: &String) -> Option<Rc<RefCell<OrderBook>>>;
}

pub struct OrderBooks {
    books: HashMap<String, Rc<RefCell<OrderBook>>>
}

impl OrderBooks {
    pub fn new(books: &[Rc<RefCell<OrderBook>>]) -> Self {
        Self {
            books: books.iter().map(|book| (book.borrow().market.symbol.clone(), book.clone())).collect()
        }
    }
}

impl OrderBookManager for OrderBooks {
    fn get_order_book(&self, symbol: &String) -> Option<Rc<RefCell<OrderBook>>> {
        let book = self.books.get(symbol);
        book.cloned()
    }
}

pub struct OrderManager {
    book_manager: Rc<dyn OrderBookManager>,
    orders: HashMap<(usize, usize), Rc<Order>>
}

impl OrderManager {
    pub fn new(book_manager: Rc<dyn OrderBookManager>) -> Self {
        Self{
            book_manager,
            orders: HashMap::new()
        }
    }

    pub fn place_order(&mut self, order: Rc<Order>, execution_policy: &impl ExecutionPolicy) -> Result<(), Box<dyn Error>> {
        if let Some(book) = self.book_manager.get_order_book(&order.market.symbol) {
            book.borrow_mut().place_order(order.clone(), execution_policy);
            self.orders.insert((order.participant_id, order.order_id), order);
            Ok(())
        }
        else {
            Err(format!("Book not found for symbol: {}", order.market.symbol).into())
        }
    }
}

pub struct LogExecutions<T> where T: ExecutionPolicy {
    policy: T
}

impl<T> LogExecutions<T> where T: ExecutionPolicy {
    pub fn new(policy: T) -> Self {
        Self { policy }
    }

    pub fn inner(&self) -> &T {
        &self.policy
    }
}

impl<T> ExecutionPolicy for LogExecutions<T> where T: ExecutionPolicy {
    fn place_order(&self, order_quantity: &OrderQuantity) -> Result<(), Box<dyn Error>>{
        if let Err(err) = self.policy.place_order(order_quantity) {
            println!("Cancel: {} on: {} Order({}:{}): {} - Reason: {}", 
                base_quantity_fmt(order_quantity.quantity, &order_quantity.order.market),
                order_quantity.order.market.symbol,
                order_quantity.order.participant_id,
                order_quantity.order.order_id,
                order_quantity.order,
                err);
            Err(err)
        }
        else {
            println!("New: {} on: {} Order({}:{}): {}", 
                base_quantity_fmt(order_quantity.quantity, &order_quantity.order.market),
                order_quantity.order.market.symbol,
                order_quantity.order.participant_id,
                order_quantity.order.order_id,
                order_quantity.order);
            Ok(())
        }
    }
    fn execute_orders(&self, executed_quantity: &mut u64, aggressor_order: &mut OrderQuantity, book_order: &mut OrderQuantity) -> Result<(), Box<dyn Error>> {
        if let Err(err) = self.policy.execute_orders(executed_quantity, aggressor_order, book_order) {
            // Execution failed/rejected - TODO: Possibly bool might not be enough, should use Result
            println!("Execution rejected");
            Err(err)
        }
        else {
            println!("Execute: {} on: {} Order({}:{}): {} Aggressor", 
                base_quantity_fmt(*executed_quantity, &aggressor_order.order.market),
                aggressor_order.order.market.symbol,
                aggressor_order.order.participant_id,
                aggressor_order.order.order_id,
                aggressor_order.order);
            println!("Execute: {} on: {} Order({}:{}): {}", 
                base_quantity_fmt(*executed_quantity, &book_order.order.market),
                book_order.order.market.symbol,
                book_order.order.participant_id,
                book_order.order.order_id,
                book_order.order);
            Ok(())
        }
    }
}

#[test]
fn test_order_book() {
    let asset_usdt = Rc::new(Asset{
        symbol: "USDT".into(),
        decimals: 2
    });

    let asset_btc = Rc::new(Asset{
        symbol: "BTC".into(),
        decimals: 7
    });
    
    let asset_eth= Rc::new(Asset{
        symbol: "ETH".into(),
        decimals: 6
    });
    
    let market_btc_usdt = Rc::new(Market {
        symbol: "BTC/USDT".into(),
        base_asset: asset_btc.clone(),
        quote_asset: asset_usdt.clone(),
        tick: 1,
        multiplier: 1,
        quote_decimals: 2,
        base_decimals: 5
    });
    
    let market_btc_eth= Rc::new(Market {
        symbol: "BTC/ETH".into(),
        base_asset: asset_btc.clone(),
        quote_asset: asset_eth.clone(),
        tick: 1,
        multiplier: 1,
        quote_decimals: 4,
        base_decimals: 5
    });

    let order_books = Rc::new(OrderBooks::new(&[
        Rc::new(RefCell::new(OrderBook::new(market_btc_usdt.clone()))),
        Rc::new(RefCell::new(OrderBook::new(market_btc_eth.clone())))
    ]));

    let mut order_manager = OrderManager::new(order_books);
    let mut margin_manager = MarginManager::new();
    margin_manager.add_participant(1001).borrow_mut().add_margin_data(&asset_btc).add_margin_data(&asset_eth).add_margin_data(&asset_usdt);
    margin_manager.add_participant(1002).borrow_mut().add_margin_data(&asset_btc).add_margin_data(&asset_eth).add_margin_data(&asset_usdt);
    //let execution_policy = LogExecutions::new(ExecuteAllways{});
    let execution_policy = LogExecutions::new(margin_manager);
    
    let orders = [
        Rc::new(Order{
            market: market_btc_usdt.clone(),
            order_id: 1,
            participant_id: 1001,
            order_data: OrderType::Limit(LimitOrder {
                side: Side::Bid,
                price: 5000000,
                quantity: 100000
            })
        }),
        Rc::new(Order{
            market: market_btc_eth.clone(),
            order_id: 2,
            participant_id: 1001,
            order_data: OrderType::Limit(LimitOrder {
                side: Side::Ask,
                price: 125000,
                quantity: 100000
            })
        }),
        Rc::new(Order{
            market: market_btc_eth.clone(),
            order_id: 3,
            participant_id: 1002,
            order_data: OrderType::Limit(LimitOrder {
                side: Side::Bid,
                price: 125000,
                quantity: 50000
            })
        })
    ];

    for order in &orders {
        if let Err(err) = order_manager.place_order(order.clone(), &execution_policy) {
            println!("Error {}", err);
        }
    }

    println!("");
    for (participant_id, margin) in execution_policy.inner().get_participants() {

        println!("Portfolio of {}", participant_id);
        println!("-------------------------------------");
        
        for (symbol, asset_data) in &margin.borrow().portfolio {
            let asset_data = asset_data.borrow();
            println!("\t{: >5} {: >10} | {: >10}", symbol,
                price_fmt(asset_data.deliver.quantity_committed, asset_data.asset.decimals),
                price_fmt(asset_data.receive.quantity_committed, asset_data.asset.decimals));
        }
    
        println!("");
    }

}
