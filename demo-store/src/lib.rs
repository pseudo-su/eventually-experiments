#![allow(missing_docs)]

mod api;
pub mod config;
pub mod order;
mod state;

use std::sync::Arc;

use eventually::aggregate::Optional;
use eventually::inmemory::{EventStoreBuilder, Projector};
// use eventually::postgres::{EventStoreBuilder, Projector};
use eventually::subscription::Transient as TransientSubscription;
use eventually::sync::RwLock;
use eventually::{AggregateRootBuilder, Repository};

use crate::config::Config;
use crate::order::{OrderAggregate, TotalOrdersProjection};


// postgresql://[user[:password]@][netloc][:port][/dbname][?param1=value1&...]
fn build_db_connection_url(
    user: &str,
    password: &str,
    netloc: &str,
    port: u16,
    database_name: &str
    // params: {}
) -> String {
    let auth_str = format!("{}:{}@", user, password);
    let server_str = format!("{}:{}", netloc, port);
    let dbname_str = format!("/{}", database_name);
    return format!("postgres://{}{}{}", auth_str, server_str, dbname_str);
}

async fn init(config: &Config) -> anyhow::Result<state::AppState> {
    // Open a connection with Postgres.
    let connection_url = build_db_connection_url(
        &config.db_username,
        &config.db_password,
        &config.db_host,
        config.db_port,
        &config.db_database,
    );

    let (mut client, connection) =
        tokio_postgres::connect(&connection_url, tokio_postgres::NoTls)
            .await
            .map_err(|err| {
                eprintln!("failed to connect to Postgres: {}", err);
                err
            })?;

    // The connection, responsible for the actual IO, must be handled by a different
    // execution context.
    eventually::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Aggregate target: in this case it's empty, but usually it would use
    // some domain services or internal repositories.
    let aggregate = OrderAggregate.as_aggregate();

    // Event store for the OrderAggregate.
    let store = EventStoreBuilder::for_aggregate(&aggregate);

    // Builder for all new AggregateRoot instances.
    let aggregate_root_builder = AggregateRootBuilder::from(aggregate);

    // Creates a Repository to read and store OrderAggregates.
    let repository = Arc::new(RwLock::new(Repository::new(
        aggregate_root_builder.clone(),
        store.clone(),
    )));

    // Create a new in-memory projection to keep the total orders computed by
    // the application.
    let total_orders_projection = Arc::new(RwLock::new(TotalOrdersProjection::default()));

    // Create an in-memory transient Subscription that starts from the very
    // beginning of the EventStream.
    let subscription = TransientSubscription::new(store.clone(), store.clone());

    // Create a new Projector for the desired projection.
    let mut total_orders_projector = Projector::new(total_orders_projection.clone(), subscription);

    // Spawn a dedicated coroutine to run the projector.
    //
    // The projector will open its own running subscription, on which
    // it will receive all oldest and newest events as they come into the EventStore,
    // and it will progressively update the projection as events arrive.
    eventually::spawn(async move { total_orders_projector.run().await.expect("should not fail") });

    return Ok(state::AppState {
        store,
        builder: aggregate_root_builder,
        repository,
        total_orders_projection,
    })
}

pub async fn serve_api(config: Config) -> anyhow::Result<()> {
    femme::with_level(config.log_level);

    let app_state = init(&config).await?;

    // Set up the HTTP router.
    let mut app = tide::new();

    app.at("/orders").nest({
        let mut api = tide::with_state(app_state);

        api.at("/history").get(api::full_history);
        api.at("/total").get(api::total_orders);

        api.at("/:id").get(api::get_order);
        api.at("/:id/create").post(api::create_order);
        api.at("/:id/add-item").post(api::add_order_item);
        api.at("/:id/complete").post(api::complete_order);
        api.at("/:id/cancel").post(api::cancel_order);
        api.at("/:id/history").get(api::history);

        api
    });

    app.listen(config.http_addr()).await?;

    Ok(())
}

pub async fn run_projector(config: Config) -> anyhow::Result<()> {
    femme::with_level(config.log_level);

    // Aggregate target: in this case it's empty, but usually it would use
    // some domain services or internal repositories.
    let aggregate = OrderAggregate.as_aggregate();

    // Event store for the OrderAggregate.
    let store = EventStoreBuilder::for_aggregate(&aggregate);

    // Create a new in-memory projection to keep the total orders computed by
    // the application.
    let total_orders_projection = Arc::new(RwLock::new(TotalOrdersProjection::default()));

    // Create an in-memory transient Subscription that starts from the very
    // beginning of the EventStream.
    let subscription = TransientSubscription::new(store.clone(), store.clone());

    // Create a new Projector for the desired projection.
    let mut total_orders_projector = Projector::new(total_orders_projection.clone(), subscription);

    // Spawn a dedicated coroutine to run the projector.
    //
    // The projector will open its own running subscription, on which
    // it will receive all oldest and newest events as they come into the EventStore,
    // and it will progressively update the projection as events arrive.
    total_orders_projector.run().await?;

    Ok(())
}
