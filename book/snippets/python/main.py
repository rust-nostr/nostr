#!/usr/bin/env python

import asyncio

from src.keys import generate, restore, vanity
from src.event.json import event_json
from src.event.builder import event_builder
from src.event.eventid import event_id
from src.event.kind import kind
from src.timestamps import timestamps
from src.event.tags import tags
from src.messages.client import client_message
from src.messages.filters import filters
from src.messages.relay import relay_message
from src.nip01 import nip01
from src.nip05 import nip05
from src.nip06 import nip06
from src.nip19 import nip19
from src.nip21 import nip21
from src.nip44 import nip44
from src.nip59 import nip59
from src.nip65 import nip65


async def main():
    generate()
    restore()
    vanity()
    event_json()
    event_builder()
    event_id()
    kind()
    timestamps()
    tags()
    client_message()
    filters()
    relay_message()
    nip01()
    await nip05()
    nip06()
    nip19()
    nip21()
    nip44()
    await nip59()
    nip65()


if __name__ == '__main__':
    asyncio.run(main())
