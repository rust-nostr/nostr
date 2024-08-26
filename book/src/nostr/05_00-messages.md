# Messages

Underpinning the Nostr Protocol is a relatively simplistic messaging system by which clients (read: applications) communicate with relays (read: databases) in order to retrieve and store data in a JSON format. This communication process is documented in more detail in [NIP-01 - Communication between clients and relays](https://github.com/nostr-protocol/nips/blob/master/01.md#communication-between-clients-and-relays) but at a very high level is broken down into three main components:

* [__Client Messages__](05_01-client-message.md) - Which define the specific formats/structure by which communication from the client to the relay is performed
* [__Filters__](05_01_01-filter.md) - Think of these as queries which can be passed to relays through client messages to help surface the relevant data for the end applications
* [__Relay Messages__](05_02-relay-message.md) - The pre-defined ways in which relays will communicate with/respond to clients

The messages themselves (for both client and relay) are passed in the form of a JSON array where the first item in the array is used to identify the type of message (e.g. "EVENT") and the subsequent items provide the relevant parameter values associated with the message in the order specified by the protocol documentation.  

Navigate to the relevant sections linked above to see the implementation of the communication rules in more detail.