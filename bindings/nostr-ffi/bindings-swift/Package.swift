// swift-tools-version:5.5
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "nostr-swift",
    platforms: [
        .macOS(.v12),
        .iOS(.v14),
    ],
    products: [
        .library(name: "Nostr", targets: ["nostrFFI", "Nostr"]),
    ],
    dependencies: [
    ],
    targets: [
        .binaryTarget(name: "nostrFFI", path: "./nostrFFI.xcframework"),
        .target(name: "Nostr", dependencies: ["nostrFFI"]),
        .testTarget(name: "NostrTests", dependencies: ["Nostr"]),
    ]
)
