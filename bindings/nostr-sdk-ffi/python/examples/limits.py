from nostr_sdk import ClientBuilder, Options, RelayLimits

# Custom relay limits
limits = RelayLimits().event_max_size(128000)

# OR, disable all limits
limits = RelayLimits.disable()

opts = Options().relay_limits(limits)
client = ClientBuilder().opts(opts).build()

# ...
