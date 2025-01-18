from nostr_sdk import ClientBuilder, Options, RelayLimits

# Custom relay limits
limits = RelayLimits().event_max_size(128000)

# OR, disable all limits
l = RelayLimits.disable()

opts = Options().relay_limits(l)
client = ClientBuilder().opts(opts).build()

# ...
