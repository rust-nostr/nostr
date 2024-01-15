#!/bin/bash

# Needed to exit from script on error
set -e

cargo install just --version 1.23.0

cd book && just test