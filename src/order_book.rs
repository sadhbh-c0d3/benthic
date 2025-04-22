use std::{cell::RefCell, cmp::min, collections::VecDeque, error::Error, rc::Rc};

use intrusive_collections::{
    intrusive_adapter, rbtree::CursorMut, Bound, KeyAdapter, RBTree, RBTreeLink,
};

use crate::{execution_policy::ExecutionPolicy, market_data_policy::MarketDataPolicy, order::*};

pub struct OrderQuantity {
    pub order: Rc<Order>,
    pub quantity: u64,
}

impl OrderQuantity {
    pub fn new_limit_order(order: Rc<Order>, limit: &LimitOrder) -> Self {
        Self {
            order: order.clone(),
            quantity: limit.quantity,
        }
    }

    pub fn new_market_order(order: Rc<Order>, market_order: &MarketOrder) -> Self {
        Self {
            order: order.clone(),
            quantity: market_order.quantity,
        }
    }
}

pub struct PriceLevel {
    pub price: u64,
    orders: RefCell<VecDeque<OrderQuantity>>,
    link: RBTreeLink,
}

intrusive_adapter!(pub PriceLevelAdapter = Rc<PriceLevel>: PriceLevel { link: RBTreeLink });

impl PriceLevel {
    pub fn new(book_order: OrderQuantity, limit: &LimitOrder) -> Self {
        Self {
            price: limit.price,
            orders: RefCell::new(vec![book_order].into()),
            link: RBTreeLink::new(),
        }
    }

    pub fn place_order(
        &self,
        mut book_order: OrderQuantity,
        execution_policy: &impl ExecutionPolicy,
        market_data_policy: &impl MarketDataPolicy,
    ) -> Result<(), Box<dyn Error>> {
        execution_policy.place_order(&mut book_order)?;
        market_data_policy.handle_order_placed(&book_order);
        self.orders.borrow_mut().push_back(book_order);
        Ok(())
    }

    pub fn match_order(
        &self,
        aggressor_order: &mut OrderQuantity,
        execution_policy: &impl ExecutionPolicy,
        market_data_policy: &impl MarketDataPolicy,
    ) -> Result<(), Box<dyn Error>> {
        let mut orders = self.orders.borrow_mut();
        while let Some(book_order) = orders.front_mut() {
            if aggressor_order.quantity == 0 {
                break;
            }
            let mut executed_quantity = min(aggressor_order.quantity, book_order.quantity);
            execution_policy.execute_orders(&mut executed_quantity, aggressor_order, book_order)?;
            market_data_policy.handle_order_executed(
                executed_quantity,
                aggressor_order,
                book_order,
            );
            if book_order.quantity == 0 {
                orders.pop_front();
            }
        }
        Ok(())
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
pub struct PriceLevels {
    levels: RBTree<PriceLevelAdapter>,
}

trait PriceLevelMatchOps {
    fn begin_ops<'a>(
        &self,
        levels: &'a mut RBTree<PriceLevelAdapter>,
    ) -> CursorMut<'a, PriceLevelAdapter>;
    fn move_next<'a>(&self, cursor: &mut CursorMut<'a, PriceLevelAdapter>);
    fn is_finished(&self, order_quantity: &OrderQuantity, level_price: u64) -> bool;
}

struct MarketMatchOps {
    book_side: Side,
}

impl MarketMatchOps {
    fn new(order_side: Side) -> Self {
        Self {
            book_side: order_side.opposite(),
        }
    }
}

impl PriceLevelMatchOps for MarketMatchOps {
    fn begin_ops<'a>(
        &self,
        levels: &'a mut RBTree<PriceLevelAdapter>,
    ) -> CursorMut<'a, PriceLevelAdapter> {
        match self.book_side {
            Side::Bid => levels.back_mut(),
            Side::Ask => levels.front_mut(),
        }
    }

    fn move_next<'a>(&self, cursor: &mut CursorMut<'a, PriceLevelAdapter>) {
        match self.book_side {
            Side::Bid => cursor.move_prev(),
            Side::Ask => cursor.move_next(),
        }
    }

    fn is_finished(&self, order_quantity: &OrderQuantity, _level_price: u64) -> bool {
        order_quantity.quantity == 0
    }
}

struct LimitMatchOps {
    book_side: Side,
    limit_price: u64,
}

impl LimitMatchOps {
    fn new(order_side: Side, limit_price: u64) -> Self {
        Self {
            book_side: order_side.opposite(),
            limit_price,
        }
    }
}

impl PriceLevelMatchOps for LimitMatchOps {
    fn begin_ops<'a>(
        &self,
        levels: &'a mut RBTree<PriceLevelAdapter>,
    ) -> CursorMut<'a, PriceLevelAdapter> {
        match self.book_side {
            Side::Bid => levels.back_mut(),
            Side::Ask => levels.front_mut(),
        }
    }

    fn move_next<'a>(&self, cursor: &mut CursorMut<'a, PriceLevelAdapter>) {
        match self.book_side {
            Side::Bid => cursor.move_prev(),
            Side::Ask => cursor.move_next(),
        }
    }

    fn is_finished(&self, order_quantity: &OrderQuantity, level_price: u64) -> bool {
        order_quantity.quantity == 0
            || match self.book_side {
                Side::Bid => level_price < self.limit_price,
                Side::Ask => level_price > self.limit_price,
            }
    }
}

impl PriceLevels {
    fn match_order_side(
        &mut self,
        order_quantity: &mut OrderQuantity,
        execution_policy: &impl ExecutionPolicy,
        market_data_policy: &impl MarketDataPolicy,
        ops: &impl PriceLevelMatchOps,
    ) -> Result<(), Box<dyn Error>> {
        let mut cursor = ops.begin_ops(&mut self.levels);

        while let Some(level) = cursor.get() {
            if ops.is_finished(order_quantity, level.price) {
                break;
            }

            level.match_order(order_quantity, execution_policy, market_data_policy)?;
            if level.is_empty() {
                cursor.remove();
            }
            ops.move_next(&mut cursor);
        }
        Ok(())
    }

    pub fn match_market_order(
        &mut self,
        order_quantity: &mut OrderQuantity,
        market_order: &MarketOrder,
        execution_policy: &impl ExecutionPolicy,
        market_data_policy: &impl MarketDataPolicy,
    ) -> Result<(), Box<dyn Error>> {
        self.match_order_side(
            order_quantity,
            execution_policy,
            market_data_policy,
            &MarketMatchOps::new(market_order.side),
        )
    }

    pub fn match_limit_order(
        &mut self,
        order_quantity: &mut OrderQuantity,
        limit: &LimitOrder,
        execution_policy: &impl ExecutionPolicy,
        market_data_policy: &impl MarketDataPolicy,
    ) -> Result<(), Box<dyn Error>> {
        self.match_order_side(
            order_quantity,
            execution_policy,
            market_data_policy,
            &LimitMatchOps::new(limit.side, limit.price),
        )
    }

    pub fn place_limit_order(
        &mut self,
        mut order_quantity: OrderQuantity,
        limit: &LimitOrder,
        execution_policy: &impl ExecutionPolicy,
        market_data_policy: &impl MarketDataPolicy,
    ) -> Result<(), Box<dyn Error>> {
        let mut cursor = self.levels.lower_bound_mut(Bound::Included(&limit.price));

        if let Some(level) = cursor.get() {
            if limit.price == level.price {
                // Level already exists: Add order to that level
                level.place_order(order_quantity, execution_policy, market_data_policy)
            } else {
                // There exists level before, which we should insert new level
                execution_policy.place_order(&mut order_quantity)?;
                market_data_policy.handle_order_placed(&order_quantity);
                cursor.insert_before(Rc::new(PriceLevel::new(order_quantity, limit)));
                Ok(())
            }
        } else {
            // We should insert at the end
            execution_policy.place_order(&mut order_quantity)?;
            market_data_policy.handle_order_placed(&order_quantity);
            cursor.insert_before(Rc::new(PriceLevel::new(order_quantity, limit)));
            Ok(())
        }
    }

    // pub fn place_stop(&mut self, order: Rc<Order>, stop: &StopOrder) {
    //     Place trigger at given level, that will place limit if triggered
    // }
}

pub struct OrderBook {
    pub market: Rc<Market>,
    bid: PriceLevels,
    ask: PriceLevels,
}

impl OrderBook {
    pub fn new(market: Rc<Market>) -> Self {
        Self {
            market,
            bid: Default::default(),
            ask: Default::default(),
        }
    }

    pub fn place_order(
        &mut self,
        order: Rc<Order>,
        execution_policy: &impl ExecutionPolicy,
        market_data_policy: &impl MarketDataPolicy,
    ) -> Result<(), Box<dyn Error>> {
        match &order.order_data {
            OrderType::Limit(limit) => {
                let mut order_quantity = OrderQuantity::new_limit_order(order.clone(), limit);
                match limit.side {
                    Side::Bid => {
                        self.ask.match_limit_order(
                            &mut order_quantity,
                            &limit,
                            execution_policy,
                            market_data_policy,
                        )?;
                        self.bid.place_limit_order(
                            order_quantity,
                            &limit,
                            execution_policy,
                            market_data_policy,
                        )
                    }
                    Side::Ask => {
                        self.bid.match_limit_order(
                            &mut order_quantity,
                            &limit,
                            execution_policy,
                            market_data_policy,
                        )?;
                        self.ask.place_limit_order(
                            order_quantity,
                            &limit,
                            execution_policy,
                            market_data_policy,
                        )
                    }
                }
            }
            OrderType::ImmediateOrCancel(limit) => {
                let mut order_quantity = OrderQuantity::new_limit_order(order.clone(), limit);
                match limit.side {
                    Side::Bid => {
                        self.ask.match_limit_order(
                            &mut order_quantity,
                            &limit,
                            execution_policy,
                            market_data_policy,
                        )
                    },
                    Side::Ask => {
                        self.bid.match_limit_order(
                            &mut order_quantity,
                            &limit,
                            execution_policy,
                            market_data_policy,
                        )
                    }
                }
            }
            OrderType::Market(market_order) => {
                let mut order_quantity =
                    OrderQuantity::new_market_order(order.clone(), market_order);
                match market_order.side {
                    Side::Bid => {
                        self.ask.match_market_order(
                            &mut order_quantity,
                            &market_order,
                            execution_policy,
                            market_data_policy,
                        )
                    }
                    Side::Ask => {
                        self.bid.match_market_order(
                            &mut order_quantity,
                            &market_order,
                            execution_policy,
                            market_data_policy,
                        )
                    }
                }
            }
            _ => Err("Invalid order type".into()),
        }
    }
}
