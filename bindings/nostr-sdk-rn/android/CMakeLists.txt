cmake_minimum_required(VERSION 3.4.1)
project(NostrSdkRn)

set (CMAKE_VERBOSE_MAKEFILE ON)
set (CMAKE_CXX_STANDARD 14)

add_library(nostr-sdk-rn            SHARED
            ../cpp/nostr-sdk-rn.cpp
            cpp-adapter.cpp
)

# Specifies a path to native header files.
include_directories(
            ../cpp
)
