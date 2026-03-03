import Foundation

public final class WebSocketClient: Sendable {
    private let lock = NSLock()

    nonisolated func connect(to urlString: String) {
        // Placeholder - full WebSocket implementation requires UIKit/AppKit context
        print("WebSocket: connecting to \(urlString)")
    }

    nonisolated func disconnect() {
        print("WebSocket: disconnected")
    }

    nonisolated func send(_ text: String) {
        print("WebSocket: sending \(text.prefix(50))...")
    }
}
