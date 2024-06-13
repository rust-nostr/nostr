#!/usr/bin/env python

import asyncio

from src.keys import generate, restore, vanity
from src.filters import filters
from src.event.json import event_json
from src.event.builder import event_builder
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
    filters()
    nip01()
    await nip05()
    nip06()
    nip19()
    nip21()
    nip44()
    nip59()
    nip65()

if __name__ == '__main__':
    asyncio.run(main())
