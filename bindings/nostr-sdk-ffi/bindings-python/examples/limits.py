from nostr_sdk import Client, Options, RelayLimits

# Custom relay limits
limits = RelayLimits().event_max_size(128000)
opts = Options().relay_limits(limits)
client = Client.with_opts(None, opts)

# ...