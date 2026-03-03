import Foundation

@Observable
final class WebSocketClient {
    private var webSocket: URLSessionWebSocketTask?
    private var session: URLSession?
    var isConnected = false
    var onEvent: ((StreamEvent) -> Void)?

    private var retryCount = 0
    private let maxRetries = 5
    private var shouldReconnect = false

    func connect(to urlString: String) {
        guard let url = URL(string: urlString) else { return }
        shouldReconnect = true
        retryCount = 0

        session = URLSession(configuration: .default)
        webSocket = session?.webSocketTask(with: url)
        webSocket?.resume()
        isConnected = true
        receiveMessage()
    }

    func disconnect() {
        shouldReconnect = false
        webSocket?.cancel(with: .normalClosure, reason: nil)
        webSocket = nil
        isConnected = false
        retryCount = 0
    }

    func send(_ text: String) {
        let message = URLSessionWebSocketTask.Message.string(text)
        webSocket?.send(message) { [weak self] error in
            if let error {
                print("WebSocket send error: \(error)")
                self?.handleDisconnect()
            }
        }
    }

    func send(_ data: Data) {
        let message = URLSessionWebSocketTask.Message.data(data)
        webSocket?.send(message) { [weak self] error in
            if let error {
                print("WebSocket send error: \(error)")
                self?.handleDisconnect()
            }
        }
    }

    private func receiveMessage() {
        webSocket?.receive { [weak self] result in
            guard let self else { return }
            switch result {
            case .success(let message):
                self.retryCount = 0
                switch message {
                case .string(let text):
                    self.handleText(text)
                case .data(let data):
                    if let text = String(data: data, encoding: .utf8) {
                        self.handleText(text)
                    }
                @unknown default:
                    break
                }
                self.receiveMessage()

            case .failure(let error):
                print("WebSocket receive error: \(error)")
                self.handleDisconnect()
            }
        }
    }

    private func handleText(_ text: String) {
        guard let data = text.data(using: .utf8) else { return }
        do {
            let event = try JSONDecoder().decode(StreamEvent.self, from: data)
            Task { @MainActor in
                onEvent?(event)
            }
        } catch {
            print("WebSocket decode error: \(error)")
        }
    }

    private func handleDisconnect() {
        isConnected = false
        guard shouldReconnect, retryCount < maxRetries else { return }

        retryCount += 1
        let delay = min(pow(2.0, Double(retryCount)), 30.0)

        Task {
            try? await Task.sleep(for: .seconds(delay))
            guard shouldReconnect else { return }
            if let urlString = webSocket?.originalRequest?.url?.absoluteString {
                connect(to: urlString)
            }
        }
    }
}
