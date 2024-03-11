from nostr_sdk import init_logger, LogLevel, NostrWalletConnectUri, Nwc

# Init logger
init_logger(LogLevel.INFO)

# Parse NWC uri
uri = NostrWalletConnectUri.parse("nostr+walletconnect://..")

# Initialize NWC client
nwc = Nwc(uri)

info = nwc.get_info()
print(info)

balance = nwc.get_balance()
print(f"Balance: {balance} SAT")
