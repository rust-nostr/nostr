#!/usr/bin/env python

from src.keys import keys
from src.event.json import event_json
from src.event.builder import event_builder
from src.nip44 import nip44
from src.vanity import vanity


def main():
    keys()
    event_json()
    event_builder()
    nip44()
    vanity()


main()
