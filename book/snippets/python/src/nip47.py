# ANCHOR: full
from nostr_sdk import NostrWalletConnectUri, Nwc, MakeInvoiceRequestParams


async def main():
    # Parse NWC uri
    uri = NostrWalletConnectUri.parse("nostr+walletconnect://..")

    # Initialize NWC client
    nwc = Nwc(uri)

    # Get info
    info = await nwc.get_info()
    print(info)

    # Get balance
    balance = await nwc.get_balance()
    print(f"Balance: {balance} SAT")

    # Pay an invoice
    await nwc.pay_invoice("lnbc..")

    # Make an invoice
    params = MakeInvoiceRequestParams(amount=100, description=None, description_hash=None, expiry=None)
    result = await nwc.make_invoice(params)
    print(f"Invoice: {result.invoice}")

# ANCHOR_END: full
