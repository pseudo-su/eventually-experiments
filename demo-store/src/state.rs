use std::sync::Arc;

use eventually::inmemory::EventStore;
use eventually::optional::AsAggregate as Optional;
use eventually::sync::RwLock;
use eventually::{AggregateRootBuilder, Repository};

use crate::order;

pub(crate) type OrderAggregate = Optional<order::OrderAggregate>;
pub(crate) type OrderStore = EventStore<String, order::OrderEvent>;
pub(crate) type OrderRepository = Repository<OrderAggregate, OrderStore>;

#[derive(Clone)]
pub(crate) struct AppState {
    pub store: OrderStore,
    pub builder: AggregateRootBuilder<OrderAggregate>,
    pub repository: Arc<RwLock<OrderRepository>>,
    pub total_orders_projection: Arc<RwLock<order::TotalOrdersProjection>>,
}
