// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "PinShot",
    platforms: [
        .macOS(.v14)
    ],
    products: [
        .executable(name: "PinShot", targets: ["PinShot"]),
        .library(name: "PinShotCore", targets: ["PinShotCore"])
    ],
    dependencies: [],
    targets: [
        .target(
            name: "PinShotCore",
            dependencies: [],
            path: "Sources/PinShotCore"
        ),
        .testTarget(
            name: "PinShotCoreTests",
            dependencies: ["PinShotCore"],
            path: "Tests/PinShotCoreTests"
        ),
        .executableTarget(
            name: "PinShot",
            dependencies: ["PinShotCore"],
            path: "Sources/PinShot"
        )
    ]
)
