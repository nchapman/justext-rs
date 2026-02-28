// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "JustextTests",
    targets: [
        .systemLibrary(
            name: "justext_uniffiFFI",
            path: "Sources/justext_uniffiFFI"
        ),
        .target(
            name: "Justext",
            dependencies: ["justext_uniffiFFI"],
            path: "Sources/Justext"
        ),
        .testTarget(
            name: "JustextTests",
            dependencies: ["Justext"],
            path: "Tests/JustextTests"
        ),
    ]
)
