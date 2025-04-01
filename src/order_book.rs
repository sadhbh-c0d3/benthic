use std::{cell::RefCell, cmp::min, collections::VecDeque, rc::Rc};

use intrusive_collections::{intrusive_adapter, rbtree::CursorMut, Bound, KeyAdapter, RBTree, RBTreeLink};

use crate::{execution_policy::ExecutionPolicy, order::*};

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

    pub fn place_order(&self, mut book_order: OrderQuantity, execution_policy: &impl ExecutionPolicy) {
        if let Ok(()) = execution_policy.place_order(&mut book_order) {
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

trait PriceLevelMatchOps {
    fn begin_ops<'a>(&self, levels: &'a mut RBTree<PriceLevelAdapter>) -> CursorMut<'a, PriceLevelAdapter>;
    fn move_next<'a>(&self, cursor: &mut CursorMut<'a, PriceLevelAdapter>);
    fn is_finished(&self, order_quantity: &OrderQuantity, price: u64) -> bool;
}

struct MarketMatchOps {
    side: Side,
}

impl MarketMatchOps {
    fn new(side: Side) -> Self {
        Self { side }
    }
}

impl PriceLevelMatchOps for MarketMatchOps {
    fn begin_ops<'a>(&self, levels: &'a mut RBTree<PriceLevelAdapter>) -> CursorMut<'a, PriceLevelAdapter> {
        match self.side {
            Side::Bid => levels.back_mut(),
            Side::Ask => levels.front_mut(),
        }
    }

    fn move_next<'a>(&self, cursor: &mut CursorMut<'a, PriceLevelAdapter>) {
        match self.side {
            Side::Bid => cursor.move_prev(),
            Side::Ask => cursor.move_next(),
        }
    }

    fn is_finished(&self, order_quantity: &OrderQuantity, _price: u64) -> bool {
        order_quantity.quantity == 0
    }
}

struct LimitMatchOps {
    side: Side,
    limit_price: u64,
}

impl LimitMatchOps {
    fn new(side: Side, limit_price: u64) -> Self {
        Self { side, limit_price }
    }
}

impl PriceLevelMatchOps for LimitMatchOps {
    fn begin_ops<'a>(&self, levels: &'a mut RBTree<PriceLevelAdapter>) -> CursorMut<'a, PriceLevelAdapter> {
        match self.side {
            Side::Bid => levels.back_mut(),
            Side::Ask => levels.front_mut(),
        }
    }

    fn move_next<'a>(&self, cursor: &mut CursorMut<'a, PriceLevelAdapter>) {
        match self.side {
            Side::Bid => cursor.move_prev(),
            Side::Ask => cursor.move_next(),
        }
    }

    fn is_finished(&self, order_quantity: &OrderQuantity, price: u64) -> bool {
        order_quantity.quantity == 0 || match self.side {
            Side::Bid => price < self.limit_price,
            Side::Ask => price > self.limit_price,
        }
    }
}

impl PriceLevels {
    fn match_order_side(
        &mut self,
        order_quantity: &mut OrderQuantity,
        execution_policy: &impl ExecutionPolicy,
        ops: &impl PriceLevelMatchOps) {
        let mut cursor = ops.begin_ops(&mut self.levels);

        while let Some(level) = cursor.get() {
            if ops.is_finished(order_quantity, level.price) {
                break;
            }

            level.match_order(order_quantity, execution_policy);
            if level.is_empty() {
                cursor.remove();
            }
            ops.move_next(&mut cursor);
        }
    }

    pub fn match_market_order(
        &mut self,
        order_quantity: &mut OrderQuantity,
        market_order: &MarketOrder,
        execution_policy: &impl ExecutionPolicy) {
        self.match_order_side(order_quantity, execution_policy, &MarketMatchOps::new(market_order.side));
    }

    pub fn match_limit_order(
        &mut self,
        order_quantity: &mut OrderQuantity,
        limit: &LimitOrder,
        execution_policy: &impl ExecutionPolicy) {
        self.match_order_side(order_quantity, execution_policy, &LimitMatchOps::new(limit.side, limit.price));
    }

    pub fn place_limit_order(&mut self, mut order_quantity: OrderQuantity, limit: &LimitOrder, execution_policy: &impl ExecutionPolicy) {
        let mut cursor = self.levels.lower_bound_mut(Bound::Included(&limit.price));

        if let Some(level) = cursor.get() {
            if limit.price == level.price {
                // Level already exists: Add order to that level
                level.place_order(order_quantity, execution_policy);
            }
            else {
                // There exists level before, which we should insert new level
                if let Ok(()) = execution_policy.place_order(&mut order_quantity) {
                    cursor.insert_before(Rc::new(PriceLevel::new(order_quantity, limit)));
                }
            }
        }
        else {
            // We should insert at the end
            if let Ok(()) = execution_policy.place_order(&mut order_quantity) {
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


