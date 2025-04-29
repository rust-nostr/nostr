# NostrMls Plan

## Principals

1. We should reduce the number of public methods on the nostr-mls library to the absolute minimum.
1. Users should never have to call `nostr_mls.storage().<method>` and use trait methods directly. Our interface should wrap those.
1. Users should never have to think about storage of either MLS or NostrMls objects, that should just happen correctly as a biproduct of their method calls.
1. The interface should be only about "getting data" and "acting". There shouldn't be any methods that directly mutate database data or manage state.

## Open Questions

1. How do we keep the MLS storage state from drifting away from the NostrMls storage state?

## State Machine of State Machines

One good way to think about this library is that it is a state machine of state machines. Every Group is a state machine that has to keep track of key material, members, admins, relays, etc., (some of that data is handled by the underlying OpenMLS storage layer and some of that data is handled by our NostrMLsStorage layer). But, fundamentally, users of the library don't need to know about any of the inner workings of the state machine or the storage layers. The state machine itself needs to be able to respond to events that are either user driven or events that are network driven. User driven events are calls to public interface methods that either trigger some change (like the create_group method) or fetch data (like the get_groups method). Network events are events that a triggered by new events arriving on subscriptions that we have opened to the wider Nostr network (for example, a new welcome message (kind 444) arriving - this would update the state, storing the correct data at the correct database layer, and then somehow update the consumer of this library of the new changes).

Using Rust's algebraic type system, we should be able to build a system that cannot get into an invalid state and only transitions from one state to another by consuming the previous state.

## Event processor thread for each group

In line with the idea of each group being it's own state machine. We'd like to create an event processor module that will allow for the creation of an individual event subscription manager and processing queue. This module would create a subscription to listen for MLS messages (kind 445) and then process those messages into messages in the database. Each group needs to have it's own processor and will subscribe to events on the group's group_relays.

## Subscription / Event Processor thread for welcome messages

Independently of groups, we need to have a subscription that listens on the user's inbox relays (and others probably) for welcome messages (kind: 444 events wrapped inside kind: 1059 events) and processes them into welcomes in the database.

## Public Interface

 - ✅ `new` create a new instance with a given storage backend

## Groups

- ✅ `get_groups` (get all groups)
- ✅ `get_group` (get a specific group by mls_group_id)
- ✅ `get_members` (get a list of member pubkeys - not included in struct because this can get very large)
- ✅ `get_relays` (get a list of group relays)
- ✅ `create_group` (create a new group)
- `leave_group` (leave a group you're in)
- `self_update` (rotate your own signing key in a group)
- `remove_member` (remove another member, only performed if you're an admin)
- `add_member` (add another member, only performed if you're an admin)

## Welcome

- ✅ `process_welcome` (what you do with a kind:444 welcome message)
- ✅ `accept_welcome` (join a group from a welcome message)
- ✅ `decline_welcome` (decline/ignore a welcome message for a group)
- ✅ `get_pending_welcomes` (get all pending welcomes)
- ✅ `get_welcome` (get a specific welcome by event_id)

## Messages

- ✅ `get_message` (get a single message in a group by mls_group_id and message event_id)
- ✅ `get_messages` (get all messages for a group by mls_group_id)
- ✅ `create_message` (create a message in a group)
- ✅ `process_message` (decrypt and process an incoming message for a group) // TODO: handle proposals and commits

## KeyPackages

- ✅ `create_key_package` (publish a new key package to relays)
- ✅ `parse_key_package` (parses a key package event to a MLS KeyPackage object)
- ✅ `delete_key_package_from_storage` (delete a key package from storage locally - does NOT send delete event to relays for key package events)
