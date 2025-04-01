use std::error::Error;

use crate::order_book::OrderQuantity;


pub trait ExecutionPolicy {
    fn place_order(&self, order_quantity: &mut OrderQuantity) -> Result<(), Box<dyn Error>>;
    fn execute_orders(&self, executed_quantity: &mut u64, aggressor_order: &mut OrderQuantity, book_order: &mut OrderQuantity) -> Result<(), Box<dyn Error>>;
}

pub struct ExecuteAllways;

impl ExecutionPolicy for ExecuteAllways {
    fn place_order(&self, book_order: &mut OrderQuantity) -> Result<(), Box<dyn Error>> {
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

