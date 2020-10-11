# demo-store

## Quick start

Start the service using a local postgres DB for the EventStore:

```sh
docker-compose up
```

## Goals

### Tech

- Postgres
- DB Migrations?
- Deployment?
- Local Dev
  - Unit tests
  - Integration tests
  - Smoke tests
- Projections to postgres tables
- GraphQL for CQRS API
- How to manipulate/reubild projections?
- How to re-run/manipulate/export/import events.

### Product

- [ ] ADMIN: Add products to store
- [ ] USER: List all products for sale
- [ ] USER: Add product to cart
- [ ] USER: Place order
- [ ] ADMIN: List all orders
- [ ] ADMIN: Dispatch order

## Build and run

```sh
docker build -t demo-store .
docker run --rm -it --init -p 8081:8080 demo-store ./demo-store-api
```
