cmake_minimum_required(VERSION 3.10)

# Project name
project(Examples)

# Add the library
add_subdirectory(../ nostr_sdk)
include_directories("../include")
link_directories("../../../target/release")

# Define the C binaries
add_executable(keys keys.c)
add_dependencies(keys nostr_sdk)
target_link_libraries(keys PRIVATE nostr_sdk_interface)

add_executable(client client.c)
add_dependencies(client nostr_sdk)
target_link_libraries(client PRIVATE nostr_sdk_interface)
