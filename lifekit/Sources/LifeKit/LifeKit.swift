import Foundation

/// LifeKit SDK — connects any app to the AgentOS memory system.
public final class LifeKit: @unchecked Sendable {

    // MARK: - Singleton

    private static let shared = LifeKit()
    private var config: LifeKitConfig?
    private var buffer: EventBuffer?
    private var httpTransport: HTTPTransport?
    private var wsTransport: WebSocketTransport?
    private var flushTask: Task<Void, Never>?

    private init() {}

    // MARK: - Public API

    /// Initialize LifeKit with an app ID and server URL.
    public static func configure(appId: String, serverURL: String, privacy: PrivacySettings = PrivacySettings()) {
        let config = LifeKitConfig(appId: appId, serverURL: serverURL, privacySettings: privacy)
        shared.config = config
        shared.buffer = EventBuffer()
        shared.httpTransport = HTTPTransport(baseURL: serverURL, appId: appId)
        shared.startPeriodicFlush()
    }

    /// Track a structured event.
    public static func track(_ event: LifeEvent) {
        guard let config = shared.config else {
            assertionFailure("LifeKit.configure() must be called before tracking events.")
            return
        }
        guard config.privacySettings.isAllowed(type: event.type) else { return }

        Task {
            await shared.buffer?.append(event)
        }
    }

    /// Convenience: track an event with type and data dictionary.
    public static func track(type: String, data: [String: Any]) {
        guard let config = shared.config else {
            assertionFailure("LifeKit.configure() must be called before tracking events.")
            return
        }
        let event = LifeEvent(
            source: config.appId,
            type: type,
            data: data.mapValues { AnyCodable($0) }
        )
        track(event)
    }

    /// Query a crystal by name.
    public static func query(_ crystalName: String) async throws -> String {
        guard let config = shared.config else {
            throw LifeKitError.notConfigured
        }
        let url = URL(string: "\(config.serverURL)/v1/crystals/\(crystalName)?appId=\(config.appId)")!
        var request = URLRequest(url: url)
        request.setValue(config.appId, forHTTPHeaderField: "X-App-Id")
        let (data, response) = try await URLSession.shared.data(for: request)
        guard let httpResponse = response as? HTTPURLResponse,
              (200...299).contains(httpResponse.statusCode) else {
            throw LifeKitError.serverError
        }
        guard let result = String(data: data, encoding: .utf8) else {
            throw LifeKitError.invalidResponse
        }
        return result
    }

    /// Listen for insight pushes from AgentOS.
    public static func listen(_ handler: @escaping (LifeInsight) -> Void) {
        guard let config = shared.config else {
            assertionFailure("LifeKit.configure() must be called before listening.")
            return
        }
        shared.wsTransport?.disconnect()
        let ws = WebSocketTransport(serverURL: config.serverURL, appId: config.appId)
        shared.wsTransport = ws
        ws.connect(onInsight: handler)
    }

    // MARK: - Internal

    private func startPeriodicFlush() {
        flushTask?.cancel()
        flushTask = Task { [weak self] in
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: 10_000_000_000) // 10 seconds
                await self?.flush()
            }
        }
    }

    private func flush() async {
        guard let buffer, let transport = httpTransport else { return }
        let events = await buffer.drain()
        guard !events.isEmpty else { return }
        do {
            let success = try await transport.upload(events: events)
            if !success {
                // Re-buffer on failure
                for event in events {
                    await buffer.append(event)
                }
            }
        } catch {
            // Re-buffer on failure
            for event in events {
                await buffer.append(event)
            }
        }
    }
}

// MARK: - Errors

public enum LifeKitError: Error, Sendable {
    case notConfigured
    case serverError
    case invalidResponse
}
