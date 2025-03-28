use std::{cell::RefCell, collections::HashMap, error::Error, rc::Rc};

use crate::{order::*, order_book::OrderQuantity, execution_policy::ExecutionPolicy};


pub struct MarginData {
    pub asset: Rc<Asset>,
    pub quantity: u64,
    pub quantity_open: u64
}

pub struct Margin {
    pub participant_id: usize,
    pub portfolio: HashMap<String, MarginData>
}

impl Margin {
    pub fn new(participant_id: usize) -> Self {
        Self {
            participant_id,
            portfolio: HashMap::new()
        }
    }

    pub fn place_order(&mut self, _order_quantity: &OrderQuantity) -> Result<(), Box<dyn Error>> {
        // TODO: Check avaliable balance/margin for open orders
        Ok(())
    }

    pub fn execute_order_begin(&mut self, executed_quantity: &mut u64, _order_quantity: &OrderQuantity) -> Result<(), Box<dyn Error>> {
        // TODO: Check avaliable balance/margin for open orders
        Ok(())
    }

    pub fn execute_order_commit(&mut self, _order_quantity: &OrderQuantity) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    pub fn execute_order_rollback(&mut self, _order_quantity: &OrderQuantity) -> Result<(), Box<dyn Error>> {
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
            else {
                Err(format!("Margin not found for {}", order_quantity.order.participant_id).into())
            }
        }
        else {
            Err("Not enough quantity".into())
        }
    }

    fn execute_orders(&self, executed_quantity: &mut u64, aggressor_order: &mut OrderQuantity, book_order: &mut OrderQuantity) -> Result<(), Box<dyn Error>> {
        if *executed_quantity > 0
        {
            let result = if let Some(aggressor_margin) = self.margins.get(&aggressor_order.order.participant_id) {
                let mut aggressor_margin_mut = aggressor_margin.borrow_mut();
                if let Ok(()) = aggressor_margin_mut.execute_order_begin(executed_quantity, aggressor_order)
                {
                    if let Some(book_margin) = self.margins.get(&book_order.order.participant_id) {
                        let mut book_margin_mut = book_margin.borrow_mut();
                        if let Ok(()) = book_margin_mut.execute_order_begin(executed_quantity, book_order)
                        {
                            if let Ok(()) = aggressor_margin_mut.execute_order_commit(&aggressor_order)
                            {
                                if let Ok(()) = book_margin_mut.execute_order_commit(&book_order) {
                                    Ok(())
                                }
                                else {
                                    if let Err(err) = aggressor_margin_mut.execute_order_rollback(&aggressor_order) {
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
