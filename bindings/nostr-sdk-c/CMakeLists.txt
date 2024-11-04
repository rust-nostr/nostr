# Minimum CMake version required
cmake_minimum_required(VERSION 3.10)

# Project Name
project(NostrSDK)

# Add your Rust library as a CMake target
add_custom_target(nostr_sdk ALL
 COMMAND cargo build --release
 WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

# Determine the library file extension based on the platform
if(WIN32)
  set(LIBRARY_FILE "nostr_sdk_c.dll")
elseif(APPLE)
  set(LIBRARY_FILE "libnostr_sdk_c.dylib")
else()
  set(LIBRARY_FILE "libnostr_sdk_c.so")
endif()

# Path to the generated header and library
set(HEADER_PATH "${CMAKE_CURRENT_SOURCE_DIR}/include")
set(LIB_PATH "${CMAKE_CURRENT_SOURCE_DIR}/../../target/release")

# Export the library's include directory
include_directories(${HEADER_PATH})
link_directories(${LIB_PATH})

add_library(nostr_sdk_interface INTERFACE)
target_include_directories(nostr_sdk_interface INTERFACE ${HEADER_PATH})
target_link_libraries(nostr_sdk_interface INTERFACE ${LIB_PATH}/${LIBRARY_FILE})

# Install header files
install(FILES ${HEADER_PATH}/nostr_sdk.h DESTINATION include)

# Install the shared library
install(FILES ${LIB_PATH}/${LIBRARY_FILE} DESTINATION lib)
