# ANCHOR: full
import asyncio
from nostr_sdk import NostrWalletConnectUri, Nwc, PayInvoiceRequest, MakeInvoiceRequest


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
    print(f"Balance: {balance} mSAT")

    # Pay an invoice
    params = PayInvoiceRequest(invoice = "lnbc..", id = None, amount = None)
    await nwc.pay_invoice(params)

    # Make an invoice
    params = MakeInvoiceRequest(amount = 100, description = None, description_hash = None, expiry = None)
    result = await nwc.make_invoice(params)
    print(f"Invoice: {result.invoice}")


if __name__ == '__main__':
   asyncio.run(main())
# ANCHOR_END: full
