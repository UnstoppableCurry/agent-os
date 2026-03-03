import Foundation

// MARK: - HTTPTransport

/// Handles batch upload of events via HTTP POST.
struct HTTPTransport: Sendable {
    let baseURL: String
    let appId: String

    /// Upload a batch of events. Returns true on success.
    func upload(events: [LifeEvent]) async throws -> Bool {
        guard !events.isEmpty else { return true }

        let url = URL(string: "\(baseURL)/v1/events")!
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.setValue(appId, forHTTPHeaderField: "X-App-Id")

        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .iso8601
        request.httpBody = try encoder.encode(events)

        let (_, response) = try await URLSession.shared.data(for: request)
        guard let httpResponse = response as? HTTPURLResponse else { return false }
        return (200...299).contains(httpResponse.statusCode)
    }
}

// MARK: - WebSocketTransport

/// Maintains a WebSocket connection for receiving insight pushes.
final class WebSocketTransport: NSObject, URLSessionWebSocketDelegate, @unchecked Sendable {
    private let serverURL: String
    private let appId: String
    private var task: URLSessionWebSocketTask?
    private var session: URLSession?
    private var onInsight: ((LifeInsight) -> Void)?
    private var isConnected = false
    private var reconnectDelay: TimeInterval = 1.0
    private let maxReconnectDelay: TimeInterval = 30.0

    init(serverURL: String, appId: String) {
        self.serverURL = serverURL
        self.appId = appId
        super.init()
    }

    func connect(onInsight: @escaping (LifeInsight) -> Void) {
        self.onInsight = onInsight
        establishConnection()
    }

    func disconnect() {
        task?.cancel(with: .goingAway, reason: nil)
        task = nil
        session?.invalidateAndCancel()
        session = nil
        isConnected = false
    }

    private func establishConnection() {
        let wsURL = serverURL
            .replacingOccurrences(of: "http://", with: "ws://")
            .replacingOccurrences(of: "https://", with: "wss://")
        guard let url = URL(string: "\(wsURL)/v1/insights?appId=\(appId)") else { return }

        session = URLSession(configuration: .default, delegate: self, delegateQueue: nil)
        task = session?.webSocketTask(with: url)
        task?.resume()
        receiveMessage()
    }

    private func receiveMessage() {
        task?.receive { [weak self] result in
            guard let self else { return }
            switch result {
            case .success(let message):
                self.handleMessage(message)
                self.receiveMessage()
            case .failure:
                self.scheduleReconnect()
            }
        }
    }

    private func handleMessage(_ message: URLSessionWebSocketTask.Message) {
        let data: Data?
        switch message {
        case .string(let text):
            data = text.data(using: .utf8)
        case .data(let d):
            data = d
        @unknown default:
            data = nil
        }
        guard let data else { return }
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        if let insight = try? decoder.decode(LifeInsight.self, from: data) {
            onInsight?(insight)
        }
    }

    private func scheduleReconnect() {
        isConnected = false
        let delay = reconnectDelay
        reconnectDelay = min(reconnectDelay * 2, maxReconnectDelay)
        DispatchQueue.global().asyncAfter(deadline: .now() + delay) { [weak self] in
            self?.establishConnection()
        }
    }

    // MARK: - URLSessionWebSocketDelegate

    func urlSession(_ session: URLSession, webSocketTask: URLSessionWebSocketTask, didOpenWithProtocol protocol: String?) {
        isConnected = true
        reconnectDelay = 1.0
    }

    func urlSession(_ session: URLSession, webSocketTask: URLSessionWebSocketTask, didCloseWith closeCode: URLSessionWebSocketTask.CloseCode, reason: Data?) {
        scheduleReconnect()
    }
}
