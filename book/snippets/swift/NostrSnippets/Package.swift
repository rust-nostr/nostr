// swift-tools-version: 5.9
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "NostrSnippets",
    platforms: [.macOS(.v13)],
    dependencies: [
        .package(url: "https://github.com/rust-nostr/nostr-sdk-swift", from:"0.36.0")
    ],
    targets: [
        .executableTarget(
            name: "NostrSnippets",
            dependencies: [
                .product(name: "NostrSDK", package: "nostr-sdk-swift"),
            ],
            path: "Sources"),
    ]
)
