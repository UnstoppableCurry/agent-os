// swift-tools-version: 5.9
// LifeKit - 连接所有 App 到 AgentOS 记忆系统的 SDK

import PackageDescription

let package = Package(
    name: "LifeKit",
    platforms: [
        .iOS(.v17),
        .macOS(.v14)
    ],
    products: [
        .library(
            name: "LifeKit",
            targets: ["LifeKit"]
        )
    ],
    targets: [
        .target(
            name: "LifeKit",
            path: "Sources/LifeKit"
        ),
        .testTarget(
            name: "LifeKitTests",
            dependencies: ["LifeKit"],
            path: "Tests/LifeKitTests"
        )
    ]
)
