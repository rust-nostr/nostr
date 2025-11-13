# Database schema

## Public keys table

- `id`: Public Key ID
- `public_key`: Public Key 32-byte array

## Lists table

- `public_key_id`: Public Key ID
- `event_kind`: The event kind of the list (i.e., 10050, 10002)
- `event_created_at`: UNIX timestamp of when the event list has been created
- `last_checked_at`: UNIX timestamp of the last check

## Relays table

- `id`: Relay ID
- `url`: Relay URL

## Relays-per-user table

- `public_key_id`: Public Key ID
- `relay_id`: Relay ID
- `bitflags`: flags of the relay (read, write, hint, etc.)
- `received_events`: number of received events from the relay for that user
- `last_received_event`: UNIX timestamp of the last received event from the relay for that user
