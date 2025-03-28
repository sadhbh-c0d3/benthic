use std::{cell::RefCell, cmp::min, collections::VecDeque, error::Error, rc::Rc};

use intrusive_collections::{intrusive_adapter, Bound, KeyAdapter, RBTree, RBTreeLink};

use crate::order::*;

pub struct OrderQuantity
{
    pub order: Rc<Order>,
    pub quantity: u64
}

impl OrderQuantity {
    pub fn new_limit_order(order: Rc<Order>, limit: &LimitOrder) -> Self {
        Self {
            order: order.clone(),
            quantity: limit.quantity
        }
    }
    
    pub fn new_market_order(order: Rc<Order>, market_order: &MarketOrder) -> Self {
        Self {
            order: order.clone(),
            quantity: market_order.quantity
        }
    }
}

pub trait ExecutionPolicy {
    fn place_order(&self, order_quantity: &OrderQuantity) -> Result<(), Box<dyn Error>>;
    fn execute_orders(&self, executed_quantity: &mut u64, aggressor_order: &mut OrderQuantity, book_order: &mut OrderQuantity) -> Result<(), Box<dyn Error>>;
}

pub struct ExecuteAllways;

impl ExecutionPolicy for ExecuteAllways {
    fn place_order(&self, book_order: &OrderQuantity) -> Result<(), Box<dyn Error>> {
        // TODO: Check available balance/margine for participant
        if book_order.quantity > 0 {
            Ok(())
        }
        else {
            Err("Not enough quantity".into())
        }
    }

    fn execute_orders(&self, executed_quantity: &mut u64, aggressor_order: &mut OrderQuantity, book_order: &mut OrderQuantity) -> Result<(), Box<dyn Error>> {
        // TODO: Check available balance/margine for each participant
        if *executed_quantity > 0 {
            aggressor_order.quantity -= *executed_quantity;
            book_order.quantity += *executed_quantity;
            Ok(())
        }
        else {
            Err("Not enough quantity".into())
        }
    }
}

pub struct PriceLevel
{
    pub price: u64,
    orders: RefCell<VecDeque<OrderQuantity>>,
    link: RBTreeLink
}

intrusive_adapter!(pub PriceLevelAdapter = Rc<PriceLevel>: PriceLevel { link: RBTreeLink });

impl PriceLevel {
    pub fn new(book_order: OrderQuantity, limit: &LimitOrder) -> Self {
        Self {
            price: limit.price,
            orders: RefCell::new(vec![book_order].into()),
            link: RBTreeLink::new()
        }
    }

    pub fn place_order(&self, book_order: OrderQuantity, execution_policy: &impl ExecutionPolicy) {
        if let Ok(()) = execution_policy.place_order(&book_order) {
            self.orders.borrow_mut().push_back(book_order);
        }
    }

    pub fn match_order(&self, aggressor_order: &mut OrderQuantity, execution_policy: &impl ExecutionPolicy) {
        let mut orders = self.orders.borrow_mut();
        while let Some(book_order) =  orders.front_mut() {
            if aggressor_order.quantity == 0 {
                break;
            }
            let mut executed_quantity = min(aggressor_order.quantity, book_order.quantity);
            drop(execution_policy.execute_orders(&mut executed_quantity, aggressor_order, book_order));
            if book_order.quantity == 0 {
                orders.pop_front();
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.orders.borrow().is_empty()
    }
}

impl<'a> KeyAdapter<'a> for PriceLevelAdapter {
    type Key = u64;
    fn get_key(&self, value: &'a PriceLevel) -> Self::Key {
        value.price
    }
}

#[derive(Default)]
pub struct PriceLevels
{
    levels: RBTree<PriceLevelAdapter>
}

impl PriceLevels {
    pub fn match_market_order(&mut self, order_quantity: &mut OrderQuantity, market_order: &MarketOrder, execution_policy: &impl ExecutionPolicy) {
        // TODO: FIX repeated code: WET Code - repeated for Bid and Ask, and then for LimitOrder
        match market_order.side {
            Side::Bid => {
                let mut cursor = self.levels.front_mut();
                while let Some(level) = cursor.get() {
                    if order_quantity.quantity > 0 {
                        break;
                    }
                    level.match_order(order_quantity, execution_policy);
                    if level.is_empty() {
                        cursor.remove();
                    }
                    cursor.move_next();
                }
            },
            Side::Ask => {
                let mut cursor = self.levels.back_mut();
                while let Some(level) = cursor.get() {
                    if order_quantity.quantity > 0 {
                        break;
                    }
                    level.match_order(order_quantity, execution_policy);
                    if level.is_empty() {
                        cursor.remove();
                    }
                    cursor.move_prev();
                }
            },
        }
    }
    
    pub fn match_limit_order(&mut self, order_quantity: &mut OrderQuantity, limit: &LimitOrder, execution_policy: &impl ExecutionPolicy) {
        // TODO: FIX repeated code: WET Code - repeated for Bid and Ask, and then for MarketOrder
        match limit.side {
            Side::Bid => {
                let mut cursor = self.levels.front_mut();
                while let Some(level) = cursor.get() {
                    if level.price > limit.price && order_quantity.quantity > 0 {
                        break;
                    }
                    level.match_order(order_quantity, execution_policy);
                    if level.is_empty() {
                        cursor.remove();
                    }
                    cursor.move_next();
                }
            },
            Side::Ask => {
                let mut cursor = self.levels.back_mut();
                while let Some(level) = cursor.get() {
                    if level.price < limit.price && order_quantity.quantity > 0 {
                        break;
                    }
                    level.match_order(order_quantity, execution_policy);
                    if level.is_empty() {
                        cursor.remove();
                    }
                    cursor.move_prev();
                }
            },
        }
    }

    pub fn place_limit_order(&mut self, order_quantity: OrderQuantity, limit: &LimitOrder, execution_policy: &impl ExecutionPolicy) {
        let mut cursor = self.levels.lower_bound_mut(Bound::Included(&limit.price));

        if let Some(level) = cursor.get() {
            if limit.price == level.price {
                // Level already exists: Add order to that level
                level.place_order(order_quantity, execution_policy);
            }
            else {
                // There exists level before, which we should insert new level
                if let Ok(()) = execution_policy.place_order(&order_quantity) {
                    cursor.insert_before(Rc::new(PriceLevel::new(order_quantity, limit)));
                }
            }
        }
        else {
            // We should insert at the end
            if let Ok(()) = execution_policy.place_order(&order_quantity) {
                cursor.insert_before(Rc::new(PriceLevel::new(order_quantity, limit)));
            }
        }
    }

    // pub fn place_stop(&mut self, order: Rc<Order>, stop: &StopOrder) {
    //     Place trigger at given level, that will place limit if triggered
    // }
}

pub struct OrderBook
{
    pub market: Rc<Market>,
    bid: PriceLevels,
    ask: PriceLevels
}

impl OrderBook {
    pub fn new(market: Rc<Market>) -> Self {
        Self {
            market,
            bid: Default::default(),
            ask: Default::default()
        }
    }

    pub fn place_order(&mut self, order: Rc<Order>, execution_policy: &impl ExecutionPolicy) {
        match &order.order_data { 
            OrderType::Limit(limit) => {
                let mut order_quantity = OrderQuantity::new_limit_order(order.clone(), limit);
                match limit.side {
                    Side::Bid => {
                        self.ask.match_limit_order(&mut order_quantity, &limit, execution_policy);
                        self.bid.place_limit_order(order_quantity, &limit, execution_policy)
                    },
                    Side::Ask => {
                        self.bid.match_limit_order(&mut order_quantity, &limit, execution_policy);
                        self.ask.place_limit_order(order_quantity, &limit, execution_policy)
                    }
                }
            },
            OrderType::ImmediateOrCancel(limit) => {
                let mut order_quantity = OrderQuantity::new_limit_order(order.clone(), limit);
                match limit.side {
                    Side::Bid => {
                        self.ask.match_limit_order(&mut order_quantity, &limit, execution_policy);
                    },
                    Side::Ask => {
                        self.bid.match_limit_order(&mut order_quantity, &limit, execution_policy);
                    }
                }
            },
            OrderType::Market(market_order) => {
                let mut order_quantity = OrderQuantity::new_market_order(order.clone(), market_order);
                match market_order.side {
                    Side::Bid => {
                        self.ask.match_market_order(&mut order_quantity, &market_order, execution_policy);
                    },
                    Side::Ask => {
                        self.bid.match_market_order(&mut order_quantity, &market_order, execution_policy);
                    }
                }
            }
        }
    }
}


