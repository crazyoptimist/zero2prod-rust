# Email Newsletter Subscription Service

This is a project from Zero To Production In Rust.

## Database Migration

Install sqlx-cli

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

Create a database

```bash
sqlx database create
```

Create a new migration

```bash
sqlx migrate add <new_migration_name>
```

Run migrations

```bash
sqlx migrate run
```

## Testing

```bash
cargo test
```

## Credits

[Zero To Production In Rust](https://www.zero2prod.com/)
