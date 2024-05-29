#!/usr/bin/env python

from src.keys import generate, restore, vanity
from src.event.json import event_json
from src.event.builder import event_builder
from src.nip01 import nip01
from src.nip05 import nip05
from src.nip06 import nip06
from src.nip19 import nip19
from src.nip21 import nip21
from src.nip44 import nip44
from src.nip59 import nip59


def main():
    generate()
    restore()
    vanity()
    event_json()
    event_builder()
    nip01()
    nip05()
    nip06()
    nip19()
    nip21()
    nip44()
    nip59()


main()
