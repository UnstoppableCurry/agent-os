import Foundation

public final class APIClient: Sendable {
    public static let shared = APIClient()

    private var baseURL: String {
        UserDefaults.standard.string(forKey: "serverURL") ?? "http://127.0.0.1:3000"
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

    // MARK: - Bots

    public func listBots() async -> [BotResponse] {
        guard let url = URL(string: "\(baseURL)/v1/bots") else { return [] }
        do {
            let (data, _) = try await URLSession.shared.data(from: url)
            let resp = try JSONDecoder().decode(ApiListResponse<BotResponse>.self, from: data)
            return resp.data ?? []
        } catch {
            print("List bots failed: \(error)")
            return []
        }
    }

    public func createBot(name: String, engine: String) async -> BotResponse? {
        guard let url = URL(string: "\(baseURL)/v1/bots") else { return nil }
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        let body: [String: String] = ["name": name, "engine": engine]
        request.httpBody = try? JSONEncoder().encode(body)

        do {
            let (data, _) = try await URLSession.shared.data(for: request)
            let resp = try JSONDecoder().decode(ApiSingleResponse<BotResponse>.self, from: data)
            return resp.data
        } catch {
            print("Create bot failed: \(error)")
            return nil
        }
    }

    public func stopBot(id: String) async -> Bool {
        guard let url = URL(string: "\(baseURL)/v1/bots/\(id)") else { return false }
        var request = URLRequest(url: url)
        request.httpMethod = "DELETE"
        do {
            let (_, response) = try await URLSession.shared.data(for: request)
            return (response as? HTTPURLResponse)?.statusCode == 200
        } catch {
            return false
        }
    }

    // MARK: - Messages (SSE streaming)

    public func sendMessage(_ content: String, botId: String, onEvent: @escaping @Sendable (StreamEvent) -> Void) async {
        guard let url = URL(string: "\(baseURL)/v1/bots/\(botId)/messages") else {
            onEvent(.error(message: "无效的 URL"))
            return
        }

        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.timeoutInterval = 120

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
            let resp = try JSONDecoder().decode(ApiSingleResponse<CrystalResponse>.self, from: data)
            return resp.data
        } catch {
            return nil
        }
    }
}

// MARK: - Response Types

struct ApiSingleResponse<T: Codable>: Codable {
    let success: Bool
    let data: T?
    let error: String?
}

struct ApiListResponse<T: Codable>: Codable {
    let success: Bool
    let data: [T]?
    let error: String?
}

public struct HealthResponse: Codable, Sendable {
    public let status: String
    public let version: String?
}

public struct CrystalResponse: Codable, Sendable {
    public let name: String?
    public let content: String?
}

public struct BotResponse: Codable, Sendable {
    public let id: String
    public let name: String
    public let engine: String
    public let state: String?
}
