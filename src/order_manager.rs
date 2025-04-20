use std::{cell::RefCell, collections::HashMap, error::Error, rc::Rc};

use crate::{
    execution_policy::ExecutionPolicy,
    market_data_policy::MarketDataPolicy,
    order::*,
    order_book::{OrderBook, OrderQuantity},
};

pub trait OrderBookManager {
    fn get_order_book(&self, symbol: &String) -> Option<Rc<RefCell<OrderBook>>>;
}

pub struct OrderBooks {
    books: HashMap<String, Rc<RefCell<OrderBook>>>,
}

impl OrderBooks {
    pub fn new(books: &[Rc<RefCell<OrderBook>>]) -> Self {
        Self {
            books: books
                .iter()
                .map(|book| (book.borrow().market.symbol.clone(), book.clone()))
                .collect(),
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
    orders: HashMap<(usize, usize), Rc<Order>>,
}

impl OrderManager {
    pub fn new(book_manager: Rc<dyn OrderBookManager>) -> Self {
        Self {
            book_manager,
            orders: HashMap::new(),
        }
    }

    pub fn place_order(
        &mut self,
        order: Rc<Order>,
        execution_policy: &impl ExecutionPolicy,
        market_data_policy: &impl MarketDataPolicy,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(book) = self.book_manager.get_order_book(&order.market.symbol) {
            book.borrow_mut()
                .place_order(order.clone(), execution_policy, market_data_policy)?;
            self.orders
                .insert((order.participant_id, order.order_id), order);
            Ok(())
        } else {
            Err(format!("Book not found for symbol: {}", order.market.symbol).into())
        }
    }
}

pub struct LogExecutions<T>
where
    T: ExecutionPolicy,
{
    policy: T,
}

impl<T> LogExecutions<T>
where
    T: ExecutionPolicy,
{
    pub fn new(policy: T) -> Self {
        Self { policy }
    }

    pub fn inner(&self) -> &T {
        &self.policy
    }
}

impl<T> ExecutionPolicy for LogExecutions<T>
where
    T: ExecutionPolicy,
{
    fn place_order(&self, order_quantity: &mut OrderQuantity) -> Result<(), Box<dyn Error>> {
        if let Err(err) = self.policy.place_order(order_quantity) {
            println!(
                "User    <--- Cancel({}):            {:24} <- (Order({}:{}): {}) - Reason: {}",
                order_quantity.order.market.symbol,
                base_quantity_fmt(order_quantity.quantity, &order_quantity.order.market),
                order_quantity.order.participant_id,
                order_quantity.order.order_id,
                order_quantity.order,
                err
            );
            Err(err)
        } else {
            println!(
                "User    <--- Promise({}):           {:24} <- (Order({}:{}): {})",
                order_quantity.order.market.symbol,
                base_quantity_fmt(order_quantity.quantity, &order_quantity.order.market),
                order_quantity.order.participant_id,
                order_quantity.order.order_id,
                order_quantity.order
            );
            Ok(())
        }
    }
    fn cancel_order(&self, order_quantity: &mut OrderQuantity) -> Result<(), Box<dyn Error>> {
        if let Err(err) = self.policy.cancel_order(order_quantity) {
            println!(
                "User    <--- Err Cancel({}):        {:24} <- (Order({}:{}): {}) - Reason: {}",
                order_quantity.order.market.symbol,
                base_quantity_fmt(order_quantity.quantity, &order_quantity.order.market),
                order_quantity.order.participant_id,
                order_quantity.order.order_id,
                order_quantity.order,
                err
            );
            Err(err)
        } else {
            println!(
                "User    <--- Cancel({}):            {:24} <- (Order({}:{}): {})",
                order_quantity.order.market.symbol,
                base_quantity_fmt(order_quantity.quantity, &order_quantity.order.market),
                order_quantity.order.participant_id,
                order_quantity.order.order_id,
                order_quantity.order
            );
            Ok(())
        }
    }
    fn execute_orders(
        &self,
        executed_quantity: &mut u64,
        aggressor_order: &mut OrderQuantity,
        book_order: &mut OrderQuantity,
    ) -> Result<(), Box<dyn Error>> {
        if let Err(err) = self
            .policy
            .execute_orders(executed_quantity, aggressor_order, book_order)
        {
            // Execution failed/rejected - TODO: Possibly bool might not be enough, should use Result
            println!("Execution rejected");
            Err(err)
        } else {
            println!(
                "User    <--- Execute({}:Aggressor): {:24} <- (Order({}:{}): {})",
                aggressor_order.order.market.symbol,
                base_quantity_fmt(*executed_quantity, &aggressor_order.order.market),
                aggressor_order.order.participant_id,
                aggressor_order.order.order_id,
                aggressor_order.order
            );
            println!(
                "User    <--- Execute({}:Book):      {:24} <- (Order({}:{}): {})",
                book_order.order.market.symbol,
                base_quantity_fmt(*executed_quantity, &book_order.order.market),
                book_order.order.participant_id,
                book_order.order.order_id,
                book_order.order
            );
            Ok(())
        }
    }
}

pub struct LogMarketData<T>
where
    T: MarketDataPolicy,
{
    policy: T,
}

impl<T> LogMarketData<T>
where
    T: MarketDataPolicy,
{
    pub fn new(policy: T) -> Self {
        Self { policy }
    }
}

impl<T> MarketDataPolicy for LogMarketData<T>
where
    T: MarketDataPolicy,
{
    fn handle_order_placed(&self, order_quantity: &OrderQuantity) {
        self.policy.handle_order_placed(order_quantity);
        println!(
            "Market   <-- Depth({}):             {:24} <- (Order({}:{}): {})",
            order_quantity.order.market.symbol,
            base_quantity_fmt(order_quantity.quantity, &order_quantity.order.market),
            order_quantity.order.participant_id,
            order_quantity.order.order_id,
            order_quantity.order
        );
    }

    fn handle_order_cancelled(&self, order_quantity: &OrderQuantity) {
        self.policy.handle_order_cancelled(order_quantity);
        println!(
            "Market   <-- Depth({}):            -{:24} <- (Order({}:{}): {})",
            order_quantity.order.market.symbol,
            base_quantity_fmt(order_quantity.quantity, &order_quantity.order.market),
            order_quantity.order.participant_id,
            order_quantity.order.order_id,
            order_quantity.order,
        );
    }

    fn handle_order_executed(
        &self,
        executed_quantity: u64,
        aggressor_order: &OrderQuantity,
        book_order: &OrderQuantity,
    ) {
        self.policy.handle_order_executed(executed_quantity, aggressor_order, book_order);
        println!(
            "Market   <-- Trade({}):             {:24} <- (Order({}:{}): {}) x (Order({}:{}): {})",
            aggressor_order.order.market.symbol,
            base_quantity_fmt(executed_quantity, &aggressor_order.order.market),
            aggressor_order.order.participant_id,
            aggressor_order.order.order_id,
            aggressor_order.order,
            book_order.order.participant_id,
            book_order.order.order_id,
            book_order.order
        );
    }
}
