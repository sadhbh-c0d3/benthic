use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    error::Error,
    rc::Rc,
};

use itertools::FoldWhile::{Continue, Done};
use itertools::Itertools;

use crate::{execution_policy::ExecutionPolicy, order::*, order_book::OrderQuantity};

pub struct MarginLotTransaction {
    /// Order of the lot owner (can be aggressor or book order)
    pub order: Rc<Order>,
    /// Price in quote currency, at which order was executed (is always quote)
    pub executed_price: u64,
    /// Quantity of the asset, of which the lot was updated (can be either base or quote)
    pub executed_quantity: u64,
    // TODO: add mark-to-market using exchange-rates, i.e. price in reporting currency
}

/// One lot on once side of an asset on asset's account for one participant account
pub struct MarginLot {
    /// Original quantity when lot was created
    pub quantity_orig: u64,
    /// Remaining quantity after matching against most recent transaction
    pub quantity_left: u64,
    /// List of all transactions that affected this lot (all quantity updates)
    pub transactions: VecDeque<MarginLotTransaction>,
}

impl MarginLot {
    /// Brand new lot
    pub fn new_with_quantity(quantity: u64) -> Self {
        Self {
            quantity_orig: quantity,
            quantity_left: quantity,
            transactions: VecDeque::new(),
        }
    }

    /// Tell how much quantity was closed so far
    pub fn get_quantity_closed(&self) -> Option<u64> {
        if self.quantity_left < self.quantity_orig {
            Some(self.quantity_orig - self.quantity_left)
        } else {
            None
        }
    }

    /// Close some quantity, and remember the transaction
    pub fn close_quantity(&mut self, quantity: u64, order: Rc<Order>, price: u64) -> Option<u64> {
        if quantity < self.quantity_left {
            self.quantity_left -= quantity;
            self.transactions.push_back(MarginLotTransaction {
                order,
                executed_price: price,
                executed_quantity: quantity,
            });
            None
        } else {
            let left = self.quantity_left;
            self.quantity_left = 0;
            self.transactions.push_back(MarginLotTransaction {
                order,
                executed_price: price,
                executed_quantity: left,
            });
            Some(quantity - left)
        }
    }
}

/// One side of and asset's account for one participant account
pub struct MarginSide {
    pub quantity_open: u64,
    pub quantity_locked: u64,
    pub quantity_committed: u64,
    pub open_lots: VecDeque<MarginLot>,
    pub closed_lots: VecDeque<MarginLot>,
}

impl MarginSide {
    /// Brand new side of an account
    pub fn new() -> Self {
        Self {
            quantity_open: 0,
            quantity_locked: 0,
            quantity_committed: 0,
            open_lots: VecDeque::new(),
            closed_lots: VecDeque::new(),
        }
    }

    /// Promise possible transaction in future (happens when you place new order on the book)
    pub fn promise_transaction(&mut self, quantity: u64) {
        self.quantity_open += quantity;
    }

    /// Cancel the promise of future transaction (either cancel or execution happened)
    pub fn cancel_transaction_promise(&mut self, quantity: u64) {
        self.quantity_open = self.quantity_open.saturating_sub(quantity);
    }

    /// Begin execution, which will produce transaction
    pub fn begin_transaction(&mut self, quantity: u64) {
        self.quantity_locked += quantity;
    }

    /// Attempt to take quantity from opposite side if available
    pub fn will_commit_opposite_side(&mut self, quantity: u64) -> Option<u64> {
        if quantity < self.quantity_committed {
            self.quantity_committed -= quantity;
            None
        } else {
            let left = quantity - self.quantity_committed;
            self.quantity_committed = 0;
            Some(left)
        }
    }

    /// Commit execution, save transaction
    pub fn commit_transaction(&mut self, unlock_quantity: u64, commit_quantity: Option<u64>) {
        self.quantity_locked -= unlock_quantity;
        commit_quantity.inspect(|x| self.quantity_committed += x);
    }

    /// Create one lot of an asset in given quantity
    pub fn create_lot(&mut self, quantity: u64, order: Rc<Order>, price: u64) {
        self.open_lots.push_back(MarginLot {
            quantity_orig: quantity,
            quantity_left: quantity,
            transactions: [MarginLotTransaction {
                order,
                executed_price: price,
                executed_quantity: quantity,
            }]
            .into(),
        });
    }

    /// Create one lot of an asset in given quantity and nootify
    pub fn create_lot_with_callback(
        &mut self,
        quantity: u64,
        order: Rc<Order>,
        price: u64,
        cb: impl FnOnce(&MarginLot),
    ) {
        self.create_lot(quantity, order, price);
        self.open_lots.back().inspect(|x| cb(*x));
    }

    /// Close lots for given quantity and tell how many were closed
    pub fn match_lots_tell(
        &mut self,
        quantity: u64,
        order: Rc<Order>,
        price: u64,
    ) -> (bool, usize, Option<u64>) {
        let result =
            self.open_lots
                .iter_mut()
                .fold_while((0, Some(quantity)), |(pos, left), lot| {
                    if let Some(left) = lot.close_quantity(left.unwrap(), order.clone(), price) {
                        Continue((pos + 1, Some(left)))
                    } else {
                        Done((pos, None))
                    }
                });
        let has_partial_match = result.is_done();
        let (pos, left) = result.into_inner();

        self.closed_lots.extend(self.open_lots.drain(..pos));

        (has_partial_match, pos, left)
    }

    /// Close lots for given quantity and notify
    pub fn match_lots_with_callback(
        &mut self,
        quantity: u64,
        order: Rc<Order>,
        price: u64,
        mut cb: impl FnMut(&MarginLot),
    ) -> Option<u64> {
        let (has_partial_match, pos, left) = self.match_lots_tell(quantity, order, price);
        if has_partial_match {
            self.open_lots.front().inspect(|lot| cb(lot));
        }
        self.closed_lots.iter().rev().skip(pos).rev().for_each(cb);
        left
    }

    /// Close lots for given quantity
    pub fn match_lots(&mut self, quantity: u64, order: Rc<Order>, price: u64) -> Option<u64> {
        let (_, _, left) = self.match_lots_tell(quantity, order, price);
        left
    }
}

/// Account of an asset for one participant's account
pub struct MarginAssetAccount {
    pub asset: Rc<Asset>,
    pub received: MarginSide,
    pub delivered: MarginSide,
}

/// Handles open and close lot events
pub trait MarginLotEventHandler {
    fn handle_lot_closed(
        &self,
        asset: Rc<Asset>,
        side: Side,
        lot: &MarginLot,
        order: Rc<Order>,
        price: u64,
    );
    fn handle_lot_opened(
        &self,
        asset: Rc<Asset>,
        side: Side,
        lot: &MarginLot,
        order: Rc<Order>,
        price: u64,
    );
}

impl MarginAssetAccount {
    pub fn new(asset: &Rc<Asset>) -> Self {
        Self {
            asset: asset.clone(),
            received: MarginSide::new(),
            delivered: MarginSide::new(),
        }
    }

    /// Promise possible receipt in future (happens when you place new order on the book)
    pub fn promise_receipt(&mut self, quantity: u64) {
        self.received.promise_transaction(quantity);
    }

    /// Promise possible delivery in future (happens when you place new order on the book)
    pub fn promise_delivery(&mut self, quantity: u64) {
        self.delivered.promise_transaction(quantity);
    }

    /// Cancel the promise of future receipt (either cancel or execution happened)
    pub fn cancel_receipt_promise(&mut self, quantity: u64) {
        self.received.cancel_transaction_promise(quantity);
    }

    /// Cancel the promise of future delivery (either cancel or execution happened)
    pub fn cancel_delivery_promise(&mut self, quantity: u64) {
        self.delivered.cancel_transaction_promise(quantity);
    }

    /// Begin receiving lot of an asset, which will produce transaction
    pub fn begin_receipt(&mut self, quantity: u64) {
        self.received.begin_transaction(quantity);
    }

    /// Begin delivering lot of an asset, which will produce transaction
    pub fn begin_delivery(&mut self, quantity: u64) {
        self.delivered.begin_transaction(quantity);
    }

    /// Commit receipt of a lot of an asset (will match existing lots on Short side)
    pub fn commit_receipt(
        &mut self,
        quantity: u64,
        order: Rc<Order>,
        price: u64,
        event_handler: &impl MarginLotEventHandler,
    ) {
        let order_2 = order.clone();
        if let Some(quantity) =
            self.delivered
                .match_lots_with_callback(quantity, order.clone(), price, |lot| {
                    event_handler.handle_lot_closed(
                        self.asset.clone(),
                        Side::Ask,
                        lot,
                        order.clone(),
                        price,
                    )
                })
        {
            self.received
                .create_lot_with_callback(quantity, order, price, |lot| {
                    event_handler.handle_lot_opened(
                        self.asset.clone(),
                        Side::Bid,
                        lot,
                        order_2,
                        price,
                    )
                });
        }
        self.received
            .commit_transaction(quantity, self.delivered.will_commit_opposite_side(quantity));
    }

    /// Commit delivery of a lot of an asset (will match existing lots on Long side)
    pub fn commit_delivery(
        &mut self,
        quantity: u64,
        order: Rc<Order>,
        price: u64,
        event_handler: &impl MarginLotEventHandler,
    ) {
        let order_2 = order.clone();
        if let Some(quantity) =
            self.received
                .match_lots_with_callback(quantity, order.clone(), price, |lot| {
                    event_handler.handle_lot_closed(
                        self.asset.clone(),
                        Side::Bid,
                        lot,
                        order.clone(),
                        price,
                    )
                })
        {
            self.delivered
                .create_lot_with_callback(quantity, order, price, |lot| {
                    event_handler.handle_lot_opened(
                        self.asset.clone(),
                        Side::Ask,
                        lot,
                        order_2,
                        price,
                    )
                });
        }
        self.delivered
            .commit_transaction(quantity, self.received.will_commit_opposite_side(quantity));
    }
}

/// Margin account of a single participant
pub struct MarginTradingAccount {
    pub account_id: usize,
    pub portfolio: HashMap<String, Rc<RefCell<MarginAssetAccount>>>,
}

impl MarginTradingAccount {
    pub fn new(account_id: usize) -> Self {
        Self {
            account_id,
            portfolio: HashMap::new(),
        }
    }

    /// Add account for an asset
    pub fn add_asset_account(&mut self, asset: &Rc<Asset>) -> &mut Self {
        self.portfolio
            .entry(asset.symbol.clone())
            .or_insert(Rc::new(RefCell::new(MarginAssetAccount::new(asset))));
        self
    }

    /// Get account for an asset
    fn get_asset_account(&self, asset: &String) -> Option<&Rc<RefCell<MarginAssetAccount>>> {
        self.portfolio.get(asset)
    }

    /// Transfer to/from account of an asset (can be deposit or withdrawal)
    pub fn transfer(&mut self, order: Rc<Order>, price: u64) -> Result<(), Box<dyn Error>> {
        if let Some(asset_account) = self.get_asset_account(&order.market.base_asset.symbol) {
            let mut asset_account_mut = asset_account.borrow_mut();
            match order.order_data {
                OrderType::Deposit(quantity) => {
                    let (base_quantity, _) = order
                        .get_quantity_and_value(quantity, price)
                        .ok_or("Mathematical overflow")?;
                    asset_account_mut.begin_receipt(base_quantity);
                    asset_account_mut.commit_receipt(base_quantity, order, price, self);
                    Ok(())
                }
                OrderType::Withdraw(quantity) => {
                    // TODO: Check available balance/margin
                    let (base_quantity, _) = order
                        .get_quantity_and_value(quantity, price)
                        .ok_or("Mathematical overflow")?;
                    asset_account_mut.begin_delivery(base_quantity);
                    asset_account_mut.commit_delivery(base_quantity, order, price, self);
                    Ok(())
                }
                _ => Err("Invalid transfer type".into()),
            }
        } else {
            Err(format!(
                "Asset account for {} not found",
                order.market.base_asset.symbol
            )
            .into())
        }
    }

    /// Account for placing an order
    pub fn place_order(&mut self, book_order: &mut OrderQuantity) -> Result<(), Box<dyn Error>> {
        // TODO: Check avaliable balance/margin for open orders

        let limit = match &book_order.order.order_data {
            OrderType::Limit(limit) => Some(limit),
            _ => None,
        }
        .ok_or("Invalid order type to place on book")?;

        let base_symbol = &book_order.order.market.base_asset.symbol;
        let quote_symbol = &book_order.order.market.quote_asset.symbol;

        if let Some(base_asset_account) = self.get_asset_account(&base_symbol) {
            if let Some(quote_asset_account) = self.get_asset_account(&quote_symbol) {
                let mut base_asset_account = base_asset_account.borrow_mut();
                let mut quote_asset_account = quote_asset_account.borrow_mut();

                let (base_quantity, quote_value) = book_order
                    .order
                    .get_quantity_and_value(book_order.quantity, limit.price)
                    .ok_or("Mathematical overflow")?;

                match limit.side {
                    Side::Ask => {
                        base_asset_account.promise_delivery(base_quantity);
                        quote_asset_account.promise_receipt(quote_value);
                    }
                    Side::Bid => {
                        base_asset_account.promise_receipt(base_quantity);
                        quote_asset_account.promise_delivery(quote_value);
                    }
                }
                Ok(())
            } else {
                Err(format!(
                    "Margin data not found for {}",
                    book_order.order.market.quote_asset.symbol
                )
                .into())
            }
        } else {
            Err(format!(
                "Margin data not found for {}",
                book_order.order.market.base_asset.symbol
            )
            .into())
        }
    }

    /// Account for cancelling an order
    pub fn cancel_order(&mut self, book_order: &mut OrderQuantity) -> Result<(), Box<dyn Error>> {
        // TODO: Check avaliable balance/margin for open orders

        let limit = match &book_order.order.order_data {
            OrderType::Limit(limit) => Some(limit),
            _ => None,
        }
        .ok_or("Invalid order type to place on book")?;

        let base_symbol = &book_order.order.market.base_asset.symbol;
        let quote_symbol = &book_order.order.market.quote_asset.symbol;

        if let Some(base_asset_account) = self.get_asset_account(&base_symbol) {
            if let Some(quote_asset_account) = self.get_asset_account(&quote_symbol) {
                let mut base_asset_account = base_asset_account.borrow_mut();
                let mut quote_asset_account = quote_asset_account.borrow_mut();

                let (base_quantity, quote_value) = book_order
                    .order
                    .get_quantity_and_value(book_order.quantity, limit.price)
                    .ok_or("Mathematical overflow")?;

                match limit.side {
                    Side::Ask => {
                        base_asset_account.cancel_delivery_promise(base_quantity);
                        quote_asset_account.cancel_receipt_promise(quote_value);
                    }
                    Side::Bid => {
                        base_asset_account.cancel_delivery_promise(base_quantity);
                        quote_asset_account.cancel_delivery_promise(quote_value);
                    }
                }
                Ok(())
            } else {
                Err(format!(
                    "Margin data not found for {}",
                    book_order.order.market.quote_asset.symbol
                )
                .into())
            }
        } else {
            Err(format!(
                "Margin data not found for {}",
                book_order.order.market.base_asset.symbol
            )
            .into())
        }
    }

    /// Begin accounting for transaction with other party
    pub fn execute_order_begin(
        &mut self,
        executed_quantity: &mut u64,
        order_quantity: &OrderQuantity,
        book_order: &OrderQuantity,
        is_aggressor: bool,
    ) -> Result<(), Box<dyn Error>> {
        // TODO: Check avaliable balance/margin for open orders

        let limit = match &book_order.order.order_data {
            OrderType::Limit(limit) => Some(limit),
            _ => None,
        }
        .ok_or("Invalid order type to place on book")?;

        let side = if is_aggressor {
            limit.side.opposite()
        } else {
            limit.side
        };
        let base_symbol = &order_quantity.order.market.base_asset.symbol;
        let quote_symbol = &order_quantity.order.market.quote_asset.symbol;

        if let Some(base_asset_account) = self.get_asset_account(&base_symbol) {
            if let Some(quote_asset_account) = self.get_asset_account(&quote_symbol) {
                let mut base_asset_account = base_asset_account.borrow_mut();
                let mut quote_asset_account = quote_asset_account.borrow_mut();

                let (base_quantity, quote_value) = order_quantity
                    .order
                    .get_quantity_and_value(*executed_quantity, limit.price)
                    .ok_or("Mathematical overflow")?;

                match side {
                    Side::Ask => {
                        if !is_aggressor {
                            base_asset_account.cancel_delivery_promise(base_quantity);
                            quote_asset_account.cancel_receipt_promise(quote_value);
                        }
                        base_asset_account.begin_delivery(base_quantity);
                        quote_asset_account.begin_receipt(quote_value);
                    }
                    Side::Bid => {
                        if !is_aggressor {
                            base_asset_account.cancel_receipt_promise(base_quantity);
                            quote_asset_account.cancel_delivery_promise(quote_value);
                        }
                        base_asset_account.begin_receipt(base_quantity);
                        quote_asset_account.begin_delivery(quote_value);
                    }
                };

                Ok(())
            } else {
                Err(format!(
                    "Margin data not found for {}",
                    order_quantity.order.market.quote_asset.symbol
                )
                .into())
            }
        } else {
            Err(format!(
                "Margin data not found for {}",
                order_quantity.order.market.base_asset.symbol
            )
            .into())
        }
    }

    /// Finish accounting and commit transaction with other party
    pub fn execute_order_commit(
        &mut self,
        executed_quantity: u64,
        order_quantity: &OrderQuantity,
        book_order: &OrderQuantity,
        is_aggressor: bool,
    ) -> Result<(), Box<dyn Error>> {
        // TODO: Unrepeat this code!

        let limit = match &book_order.order.order_data {
            OrderType::Limit(limit) => Some(limit),
            _ => None,
        }
        .ok_or("Invalid order type to place on book")?;

        let side = if is_aggressor {
            limit.side.opposite()
        } else {
            limit.side
        };
        let base_symbol = &order_quantity.order.market.base_asset.symbol;
        let quote_symbol = &order_quantity.order.market.quote_asset.symbol;

        if let Some(base_asset_account) = self.get_asset_account(base_symbol) {
            if let Some(quote_asset_account) = self.get_asset_account(quote_symbol) {
                let mut base_asset_account = base_asset_account.borrow_mut();
                let mut quote_asset_account = quote_asset_account.borrow_mut();

                let (base_quantity, quote_value) = order_quantity
                    .order
                    .get_quantity_and_value(executed_quantity, limit.price)
                    .ok_or("Mathematical overflow")?;

                match side {
                    Side::Ask => {
                        base_asset_account.commit_delivery(
                            base_quantity,
                            order_quantity.order.clone(),
                            limit.price,
                            self,
                        );
                        quote_asset_account.commit_receipt(
                            quote_value,
                            order_quantity.order.clone(),
                            limit.price,
                            self,
                        );
                    }
                    Side::Bid => {
                        base_asset_account.commit_receipt(
                            base_quantity,
                            order_quantity.order.clone(),
                            limit.price,
                            self,
                        );
                        quote_asset_account.commit_delivery(
                            quote_value,
                            order_quantity.order.clone(),
                            limit.price,
                            self,
                        );
                    }
                };

                Ok(())
            } else {
                Err(format!(
                    "Margin data not found for {}",
                    order_quantity.order.market.quote_asset.symbol
                )
                .into())
            }
        } else {
            Err(format!(
                "Margin data not found for {}",
                order_quantity.order.market.base_asset.symbol
            )
            .into())
        }
    }

    /// Possibly support rollback
    pub fn execute_order_rollback(
        &mut self,
        _executed_quantity: u64,
        _order_quantity: &OrderQuantity,
    ) -> Result<(), Box<dyn Error>> {
        // TODO: Undo the the commit - What if rollback fails? ¯\_(ツ)_/¯
        Ok(())
    }
}

impl MarginLotEventHandler for MarginTradingAccount {
    fn handle_lot_closed(
        &self,
        asset: Rc<Asset>,
        side: Side,
        lot: &MarginLot,
        order: Rc<Order>,
        price: u64,
    ) {
        println!(
            "Margin   <-- Lot({}:{}): close {:28}    <- (Order({}:{}): {} at {})",
            self.account_id,
            asset.symbol,
            format!(
                "{:6} {:10} ({})",
                lot_side(side),
                price_fmt(lot.quantity_left, asset.decimals),
                price_fmt(lot.quantity_orig, asset.decimals)
            ),
            order.participant_id,
            order.order_id,
            order,
            quote_price_fmt(price, &order.market)
        )
    }

    fn handle_lot_opened(
        &self,
        asset: Rc<Asset>,
        side: Side,
        lot: &MarginLot,
        order: Rc<Order>,
        price: u64,
    ) {
        println!(
            "Margin   <-- Lot({}:{}):  open {:28}    <- (Order({}:{}): {} at {})",
            self.account_id,
            asset.symbol,
            format!(
                "{:6} {:10}",
                lot_side(side),
                price_fmt(lot.quantity_orig, asset.decimals)
            ),
            order.participant_id,
            order.order_id,
            order,
            quote_price_fmt(price, &order.market)
        )
    }
}

/// Manager of all Margin accounts
pub struct MarginManager {
    margins: HashMap<usize, Rc<RefCell<MarginTradingAccount>>>,
}

impl MarginManager {
    pub fn new() -> Self {
        Self {
            margins: HashMap::new(),
        }
    }

    pub fn add_account(&mut self, participant_id: usize) -> &Rc<RefCell<MarginTradingAccount>> {
        self.margins
            .entry(participant_id)
            .or_insert(Rc::new(RefCell::new(MarginTradingAccount::new(
                participant_id,
            ))))
    }

    pub fn get_participants(&self) -> &HashMap<usize, Rc<RefCell<MarginTradingAccount>>> {
        &self.margins
    }
}

impl ExecutionPolicy for MarginManager {
    /// Perform margin checks and accounting for new order placement
    fn place_order(&self, order_quantity: &mut OrderQuantity) -> Result<(), Box<dyn Error>> {
        if order_quantity.quantity > 0 {
            if let Some(margin) = self.margins.get(&order_quantity.order.participant_id) {
                margin.borrow_mut().place_order(order_quantity)
            } else {
                Err(format!(
                    "Margin not found for {}",
                    order_quantity.order.participant_id
                )
                .into())
            }
        } else {
            Err("Not enough quantity".into())
        }
    }

    /// Perform margin checks and accounting for order cancel
    fn cancel_order(&self, order_quantity: &mut OrderQuantity) -> Result<(), Box<dyn Error>> {
        if order_quantity.quantity > 0 {
            if let Some(margin) = self.margins.get(&order_quantity.order.participant_id) {
                margin.borrow_mut().cancel_order(order_quantity)
            } else {
                Err(format!(
                    "Margin not found for {}",
                    order_quantity.order.participant_id
                )
                .into())
            }
        } else {
            Err("Not enough quantity".into())
        }
    }

    /// Perform margin checks and accounting for order execution and store transaction record
    fn execute_orders(
        &self,
        executed_quantity: &mut u64,
        aggressor_order: &mut OrderQuantity,
        book_order: &mut OrderQuantity,
    ) -> Result<(), Box<dyn Error>> {
        if *executed_quantity > 0 {
            let result = if let Some(aggressor_margin) =
                self.margins.get(&aggressor_order.order.participant_id)
            {
                let mut aggressor_margin_mut = aggressor_margin.borrow_mut();
                if let Ok(()) = aggressor_margin_mut.execute_order_begin(
                    executed_quantity,
                    aggressor_order,
                    &book_order,
                    true,
                ) {
                    if let Some(book_margin) = self.margins.get(&book_order.order.participant_id) {
                        let mut book_margin_mut = book_margin.borrow_mut();
                        if let Ok(()) = book_margin_mut.execute_order_begin(
                            executed_quantity,
                            book_order,
                            &book_order,
                            false,
                        ) {
                            if let Ok(()) = aggressor_margin_mut.execute_order_commit(
                                *executed_quantity,
                                &aggressor_order,
                                &book_order,
                                true,
                            ) {
                                if let Ok(()) = book_margin_mut.execute_order_commit(
                                    *executed_quantity,
                                    &book_order,
                                    &book_order,
                                    false,
                                ) {
                                    Ok(())
                                } else {
                                    if let Err(err) = aggressor_margin_mut.execute_order_rollback(
                                        *executed_quantity,
                                        &aggressor_order,
                                    ) {
                                        Err(err)
                                    } else {
                                        Err(format!(
                                            "Margin failed commit execution for {}",
                                            book_order.order.participant_id
                                        )
                                        .into())
                                    }
                                }
                            } else {
                                Err(format!(
                                    "Margin failed commit execute for {}",
                                    book_order.order.participant_id
                                )
                                .into())
                            }
                        } else {
                            Err(format!(
                                "Margin failed begin execute for {}",
                                book_order.order.participant_id
                            )
                            .into())
                        }
                    } else {
                        Err(
                            format!("Margin not found for {}", book_order.order.participant_id)
                                .into(),
                        )
                    }
                } else {
                    Err(format!(
                        "Margin failed begin execute for {}",
                        aggressor_order.order.participant_id
                    )
                    .into())
                }
            } else {
                Err(format!(
                    "Margin not found for {}",
                    aggressor_order.order.participant_id
                )
                .into())
            };

            if let Err(err) = result {
                Err(err)
            } else {
                aggressor_order.quantity -= *executed_quantity;
                book_order.quantity += *executed_quantity;
                Ok(())
            }
        } else {
            Err("Not enough quantity".into())
        }
    }
}
