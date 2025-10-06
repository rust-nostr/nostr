# Database schema

## Public keys table

- `id`: Public Key ID
- `public_key`: Public Key 32-byte array

## Lists table

- `event_created_at`: UNIX timestamp
- `last_checked_at`: UNIX timestamp of the last check

## Relays table

- `id`: Relay ID
- `url`: Relay URL
