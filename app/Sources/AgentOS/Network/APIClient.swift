import Foundation

public final class APIClient: Sendable {
    public static let shared = APIClient()

    private var baseURL: String {
        UserDefaults.standard.string(forKey: "serverURL") ?? "http://localhost:3000"
    }

    // MARK: - Health

    public func getHealth() async -> HealthResponse? {
        guard let url = URL(string: "\(baseURL)/health") else { return nil }
        do {
            let (data, _) = try await URLSession.shared.data(from: url)
            return try JSONDecoder().decode(HealthResponse.self, from: data)
        } catch {
            print("Health check failed: \(error)")
            return nil
        }
    }

    // MARK: - Messages (streaming)

    public func sendMessage(_ content: String, botId: String?, onEvent: @escaping @Sendable (StreamEvent) -> Void) async {
        guard let url = URL(string: "\(baseURL)/v1/bots/\(botId ?? "default")/messages") else {
            onEvent(.error(message: "Invalid URL"))
            return
        }

        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")

        let body: [String: String] = ["content": content]
        request.httpBody = try? JSONEncoder().encode(body)

        do {
            let (bytes, _) = try await URLSession.shared.bytes(for: request)
            for try await line in bytes.lines {
                if line.hasPrefix("data: ") {
                    let jsonStr = String(line.dropFirst(6))
                    if jsonStr == "[DONE]" {
                        onEvent(.messageStop(stopReason: "end_turn"))
                        return
                    }
                    if let data = jsonStr.data(using: .utf8),
                       let event = try? JSONDecoder().decode(StreamEvent.self, from: data) {
                        onEvent(event)
                    }
                }
            }
        } catch {
            onEvent(.error(message: error.localizedDescription))
        }
    }

    // MARK: - Crystals

    public func getCrystal(_ name: String) async -> CrystalResponse? {
        guard let url = URL(string: "\(baseURL)/v1/memory/crystals/\(name)") else { return nil }
        do {
            let (data, _) = try await URLSession.shared.data(from: url)
            return try JSONDecoder().decode(CrystalResponse.self, from: data)
        } catch {
            return nil
        }
    }

    // MARK: - Bots

    public func createBot(name: String, engine: String) async -> BotResponse? {
        guard let url = URL(string: "\(baseURL)/v1/bots") else { return nil }
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        let body: [String: String] = ["name": name, "engine": engine]
        request.httpBody = try? JSONEncoder().encode(body)

        do {
            let (data, _) = try await URLSession.shared.data(for: request)
            return try JSONDecoder().decode(BotResponse.self, from: data)
        } catch {
            return nil
        }
    }
}

// MARK: - Response Types

public struct HealthResponse: Codable, Sendable {
    public let status: String
    public let version: String?
}

public struct CrystalResponse: Codable, Sendable {
    public let name: String
    public let content: String?
}

public struct BotResponse: Codable, Sendable {
    public let id: String
    public let name: String
    public let engine: String
}
