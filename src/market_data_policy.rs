use crate::order_book::OrderQuantity;

pub trait MarketDataPolicy {
    fn handle_order_placed(&self, order_quantity: &OrderQuantity);
    fn handle_order_cancelled(&self, order_quantity: &OrderQuantity);
    fn handle_order_executed(
        &self,
        executed_quantity: u64,
        aggressor_order: &OrderQuantity,
        book_order: &OrderQuantity,
    );
}

pub struct MarketDataNull;

impl MarketDataPolicy for MarketDataNull {
    fn handle_order_placed(&self, _order_quantity: &OrderQuantity) {}
    fn handle_order_cancelled(&self, _order_quantity: &OrderQuantity) {}
    fn handle_order_executed(
        &self,
        _executed_quantity: u64,
        _aggressor_order: &OrderQuantity,
        _book_order: &OrderQuantity,
    ) {
    }
}
