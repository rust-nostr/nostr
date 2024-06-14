#!/usr/bin/env python

from setuptools import setup
from pathlib import Path

this_directory = Path(__file__).parent
long_description = (this_directory / "README.md").read_text()

setup(
    name='nostr-sdk',
    version='0.32.2',
    description="High level Nostr client library.",
    long_description=long_description,
    long_description_content_type='text/markdown',
    include_package_data=True,
    zip_safe=False,
    packages=['nostr_sdk'],
    package_dir={'nostr_sdk': './src/nostr-sdk'},
    url="https://github.com/rust-nostr/nostr",
    author="Yuki Kishimoto <yukikishimoto@protonmail.com>",
    license="MIT",
    # This is required to ensure the library name includes the python version, abi, and platform tags
    # See issue #350 for more information
    has_ext_modules=lambda: True,
)
