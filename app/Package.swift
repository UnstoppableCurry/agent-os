// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "AgentOS",
    platforms: [.iOS(.v17), .macOS(.v14)],
    products: [
        .library(name: "AgentOS", targets: ["AgentOS"])
    ],
    targets: [
        .target(
            name: "AgentOS",
            path: "Sources/AgentOS"
        )
    ]
)
