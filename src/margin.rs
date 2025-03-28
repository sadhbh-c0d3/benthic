use std::{cell::RefCell, collections::HashMap, error::Error, rc::Rc};

use crate::{execution_policy::ExecutionPolicy, order::*, order_book::OrderQuantity};

pub struct MarginLot {
    pub quantity_orig: u64,
    pub quantity_left: u64,
}

pub struct MarginSide {
    pub quantity_open: u64,
    pub quantity_locked: u64,
    pub quantity_committed: u64,
    pub lots: Vec<MarginLot> // TODO: Match executions against the lots
}

pub struct MarginAsset {
    pub asset: Rc<Asset>,
    pub receive: MarginSide,
    pub deliver: MarginSide
}

pub struct Margin {
    pub participant_id: usize,
    pub portfolio: HashMap<String, Rc<RefCell<MarginAsset>>>
}

impl Margin {
    pub fn new(participant_id: usize) -> Self {
        Self {
            participant_id,
            portfolio: HashMap::new()
        }
    }

    fn get_margin_data_mut(&self, asset: &String) -> Option<&Rc<RefCell<MarginAsset>>> {
        self.portfolio.get(asset)
    }

    pub fn place_order(&mut self, order_quantity: &OrderQuantity) -> Result<(), Box<dyn Error>> {
        // TODO: Check avaliable balance/margin for open orders

        if let Some(base_margin_data) = self.get_margin_data_mut(&order_quantity.order.market.base_asset.symbol) {
            if let Some(quote_margin_data) = self.get_margin_data_mut(&order_quantity.order.market.quote_asset.symbol) {
                
                match &order_quantity.order.order_data {
                    OrderType::Limit(limit) => {
                        
                        let mut base_margin_data_mut = base_margin_data.borrow_mut();
                        let mut quote_margin_data_mut = quote_margin_data.borrow_mut();

                        let order_value = calculate_value(
                                    order_quantity.quantity,
                                    limit.price, 
                                    order_quantity.order.market.base_decimals,
                                    order_quantity.order.market.quote_decimals);
                        
                        // TODO: Move this logic into MarginData
                        match limit.side {
                            Side::Ask => {
                                base_margin_data_mut.deliver.quantity_open += order_quantity.quantity;
                                quote_margin_data_mut.receive.quantity_open += order_value;
                            },
                            Side::Bid => {
                                base_margin_data_mut.receive.quantity_open += order_quantity.quantity;
                                quote_margin_data_mut.deliver.quantity_open += order_value;
                            }
                        }
                        Ok(())
                    }
                    _ => Err(format!("Invalid order type to place on book").into())
                }
            }
            else { Err(format!("Margin data not found for {}", order_quantity.order.market.quote_asset.symbol).into()) }
        }
        else { Err(format!("Margin data not found for {}", order_quantity.order.market.base_asset.symbol).into()) }
    }

    pub fn execute_order_begin(&mut self, executed_quantity: &mut u64, order_quantity: &OrderQuantity, is_aggressor: bool) -> Result<(), Box<dyn Error>> {
        // TODO: Check avaliable balance/margin for open orders

        if let Some(base_margin_data) = self.get_margin_data_mut(&order_quantity.order.market.base_asset.symbol) {
            if let Some(quote_margin_data) = self.get_margin_data_mut(&order_quantity.order.market.quote_asset.symbol) {

                if let Some(limit) = match &order_quantity.order.order_data {
                    OrderType::Limit(limit1) => Some(limit1),
                    OrderType::ImmediateOrCancel(limit1) => Some(limit1),
                    OrderType::Market(_market_order) => None
                } {
                    let mut base_margin_data_mut = base_margin_data.borrow_mut();
                    let mut quote_margin_data_mut = quote_margin_data.borrow_mut();

                    let order_value = calculate_value(
                            *executed_quantity,
                            limit.price, 
                            order_quantity.order.market.base_decimals,
                            order_quantity.order.market.quote_decimals);
                    
                    // TODO: Move this logic into MarginData
                    // base_margin_data_mut.match_and_lock_lots(limit.side, *executed_quantity);
                    // quote_margin_data_mut.match_and_lock_lots(limit.side.opposite(), order_value);

                    match limit.side {
                        Side::Ask => {
                            base_margin_data_mut.deliver.quantity_locked += *executed_quantity;
                            quote_margin_data_mut.receive.quantity_locked += order_value;

                            if !is_aggressor {
                                base_margin_data_mut.deliver.quantity_open -= *executed_quantity;
                                quote_margin_data_mut.receive.quantity_open -= order_value;
                            }
                        },
                        Side::Bid => {
                            base_margin_data_mut.receive.quantity_locked += *executed_quantity;
                            quote_margin_data_mut.deliver.quantity_locked += order_value;

                            if !is_aggressor {
                                base_margin_data_mut.receive.quantity_open -= *executed_quantity;
                                quote_margin_data_mut.deliver.quantity_open -= order_value;
                            }
                        }
                    };
                    
                    Ok(())

                } else { Err("Unsupported order type".into())} // TODO: Change parameter type from OrderQuantity to new type Execution with price and side in it
            }
            else { Err(format!("Margin data not found for {}", order_quantity.order.market.quote_asset.symbol).into()) }
        }
        else { Err(format!("Margin data not found for {}", order_quantity.order.market.base_asset.symbol).into()) }
    }

    pub fn execute_order_commit(&mut self, executed_quantity: u64, order_quantity: &OrderQuantity) -> Result<(), Box<dyn Error>> {
        // TODO: Unrepeat this code!

        if let Some(base_margin_data) = self.get_margin_data_mut(&order_quantity.order.market.base_asset.symbol) {
            if let Some(quote_margin_data) = self.get_margin_data_mut(&order_quantity.order.market.quote_asset.symbol) {

                if let Some(limit) = match &order_quantity.order.order_data {
                    OrderType::Limit(limit1) => Some(limit1),
                    OrderType::ImmediateOrCancel(limit1) => Some(limit1),
                    OrderType::Market(_market_order) => None
                } {
                    let mut base_margin_data_mut = base_margin_data.borrow_mut();
                    let mut quote_margin_data_mut = quote_margin_data.borrow_mut();

                    let order_value = calculate_value(
                            executed_quantity,
                            limit.price, 
                            order_quantity.order.market.base_decimals,
                            order_quantity.order.market.quote_decimals);
                    
                    // TODO: Move this logic into MarginData
                    // base_margin_data_mut.match_and_commmit_lots(limit.side, *executed_quantity);
                    // quote_margin_data_mut.match_and_commit_lots(limit.side.opposite(), order_value);
                    
                    match limit.side {
                        Side::Ask => {
                            base_margin_data_mut.deliver.quantity_committed += executed_quantity;
                            quote_margin_data_mut.receive.quantity_committed += order_value;

                            base_margin_data_mut.deliver.quantity_locked -= executed_quantity;
                            quote_margin_data_mut.receive.quantity_locked -= order_value;
                        },
                        Side::Bid => {
                            base_margin_data_mut.receive.quantity_committed += executed_quantity;
                            quote_margin_data_mut.deliver.quantity_committed += order_value;

                            base_margin_data_mut.receive.quantity_locked -= executed_quantity;
                            quote_margin_data_mut.deliver.quantity_locked -= order_value;
                        }
                    };
                    
                    Ok(())

                } else { Err("Unsupported order type".into())} // TODO: Change parameter type from OrderQuantity to new type Execution with price and side in it
            }
            else { Err(format!("Margin data not found for {}", order_quantity.order.market.quote_asset.symbol).into()) }
        }
        else { Err(format!("Margin data not found for {}", order_quantity.order.market.base_asset.symbol).into()) }
    }

    pub fn execute_order_rollback(&mut self, _executed_quantity: u64, _order_quantity: &OrderQuantity) -> Result<(), Box<dyn Error>> {
        // TODO: Undo the the commit - What if rollback fails? ¯\_(ツ)_/¯
        Ok(())
    }
}

pub struct MarginManager {
    margins: HashMap<usize, Rc<RefCell<Margin>>>
}

impl MarginManager {
    pub fn new() -> Self {
        Self {
            margins: HashMap::new()
        }
    }

    pub fn add_participant(&mut self, participant_id: usize) {
        self.margins.entry(participant_id).or_insert(Rc::new(RefCell::new(Margin::new(participant_id))));
    }
}

impl ExecutionPolicy for MarginManager {
    fn place_order(&self, order_quantity: &OrderQuantity) -> Result<(), Box<dyn Error>> {
        if order_quantity.quantity > 0 {
            if let Some(margin) = self.margins.get(&order_quantity.order.participant_id) {
                margin.borrow_mut().place_order(order_quantity)
            }
            else { Err(format!("Margin not found for {}", order_quantity.order.participant_id).into()) }
        }
        else { Err("Not enough quantity".into()) }
    }

    fn execute_orders(&self, executed_quantity: &mut u64, aggressor_order: &mut OrderQuantity, book_order: &mut OrderQuantity) -> Result<(), Box<dyn Error>> {
        if *executed_quantity > 0
        {
            let result = if let Some(aggressor_margin) = self.margins.get(&aggressor_order.order.participant_id) {
                let mut aggressor_margin_mut = aggressor_margin.borrow_mut();
                if let Ok(()) = aggressor_margin_mut.execute_order_begin(executed_quantity, aggressor_order, true)
                {
                    if let Some(book_margin) = self.margins.get(&book_order.order.participant_id) {
                        let mut book_margin_mut = book_margin.borrow_mut();
                        if let Ok(()) = book_margin_mut.execute_order_begin(executed_quantity, book_order, false)
                        {
                            if let Ok(()) = aggressor_margin_mut.execute_order_commit(*executed_quantity, &aggressor_order)
                            {
                                if let Ok(()) = book_margin_mut.execute_order_commit(*executed_quantity, &book_order) {
                                    Ok(())
                                }
                                else {
                                    if let Err(err) = aggressor_margin_mut.execute_order_rollback(*executed_quantity,&aggressor_order) {
                                        Err(err)
                                    }
                                    else { Err(format!("Margin failed commit execution for {}", book_order.order.participant_id).into()) }
                                }
                            }
                            else { Err(format!("Margin failed commit execute for {}", book_order.order.participant_id).into()) }
                        }
                        else { Err(format!("Margin failed begin execute for {}", book_order.order.participant_id).into()) }
                    }
                    else { Err(format!("Margin not found for {}", book_order.order.participant_id).into()) }
                }
                else { Err(format!("Margin failed begin execute for {}", aggressor_order.order.participant_id).into()) }
            }
            else { Err(format!("Margin not found for {}", aggressor_order.order.participant_id).into()) };
            
            if let Err(err) = result {
                Err(err)
            }
            else {
                aggressor_order.quantity -= *executed_quantity;
                book_order.quantity += *executed_quantity;
                Ok(())
            }
        }
        else {
            Err("Not enough quantity".into())
        }
    }
}
