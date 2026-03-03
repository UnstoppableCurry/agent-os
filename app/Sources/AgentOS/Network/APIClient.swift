import Foundation

actor APIClient {
    static let shared = APIClient()

    private var baseURL: String {
        UserDefaults.standard.string(forKey: "serverURL") ?? "http://localhost:3000"
    }

    private let decoder = JSONDecoder()
    private let encoder = JSONEncoder()

    // MARK: - Health

    func getHealth() async -> HealthResponse? {
        guard let url = URL(string: "\(baseURL)/health") else { return nil }
        do {
            let (data, _) = try await URLSession.shared.data(from: url)
            return try decoder.decode(HealthResponse.self, from: data)
        } catch {
            print("Health check failed: \(error)")
            return nil
        }
    }

    // MARK: - Sessions

    func createSession(botId: String? = nil) async -> SessionResponse? {
        guard let url = URL(string: "\(baseURL)/v1/sessions") else { return nil }
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")

        var body: [String: Any] = [:]
        if let botId { body["botId"] = botId }
        request.httpBody = try? JSONSerialization.data(withJSONObject: body)

        do {
            let (data, _) = try await URLSession.shared.data(for: request)
            return try decoder.decode(SessionResponse.self, from: data)
        } catch {
            print("Create session failed: \(error)")
            return nil
        }
    }

    // MARK: - Messages (SSE streaming)

    func sendMessage(_ content: String, botId: String?, onEvent: @escaping (StreamEvent) -> Void) async {
        // Create session first if needed
        guard let session = await createSession(botId: botId) else {
            onEvent(.error(message: "Failed to create session"))
            return
        }

        guard let url = URL(string: "\(baseURL)/v1/sessions/\(session.id)/messages") else {
            onEvent(.error(message: "Invalid URL"))
            return
        }

        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.setValue("text/event-stream", forHTTPHeaderField: "Accept")

        let body: [String: Any] = ["content": content, "stream": true]
        request.httpBody = try? JSONSerialization.data(withJSONObject: body)

        do {
            let (bytes, _) = try await URLSession.shared.bytes(for: request)
            var buffer = ""

            for try await byte in bytes {
                let char = String(UnicodeScalar(byte))
                buffer += char

                if buffer.hasSuffix("\n\n") {
                    parseSSEBuffer(buffer, onEvent: onEvent)
                    buffer = ""
                }
            }
            if !buffer.isEmpty {
                parseSSEBuffer(buffer, onEvent: onEvent)
            }
        } catch {
            onEvent(.error(message: error.localizedDescription))
        }
    }

    private func parseSSEBuffer(_ buffer: String, onEvent: @escaping (StreamEvent) -> Void) {
        let lines = buffer.components(separatedBy: "\n")
        for line in lines {
            if line.hasPrefix("data: ") {
                let jsonStr = String(line.dropFirst(6))
                if jsonStr == "[DONE]" {
                    onEvent(.messageStop(stopReason: "end_turn"))
                    return
                }
                if let data = jsonStr.data(using: .utf8),
                   let event = try? decoder.decode(StreamEvent.self, from: data) {
                    onEvent(event)
                }
            }
        }
    }

    // MARK: - Crystals

    func getCrystal(_ name: String) async -> CrystalResponse? {
        guard let url = URL(string: "\(baseURL)/v1/crystals/\(name)") else { return nil }
        do {
            let (data, _) = try await URLSession.shared.data(from: url)
            return try decoder.decode(CrystalResponse.self, from: data)
        } catch {
            print("Get crystal failed: \(error)")
            return nil
        }
    }

    // MARK: - Bots

    func createBot(name: String, engine: String) async -> BotResponse? {
        guard let url = URL(string: "\(baseURL)/v1/bots") else { return nil }
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")

        let body: [String: String] = ["name": name, "engine": engine]
        request.httpBody = try? encoder.encode(body)

        do {
            let (data, _) = try await URLSession.shared.data(for: request)
            return try decoder.decode(BotResponse.self, from: data)
        } catch {
            print("Create bot failed: \(error)")
            return nil
        }
    }
}

// MARK: - Response Types

struct HealthResponse: Codable {
    let status: String
    let version: String?
}

struct SessionResponse: Codable {
    let id: String
    let createdAt: String?
}

struct CrystalResponse: Codable {
    let name: String
    let data: [String: String]?
}

struct BotResponse: Codable {
    let id: String
    let name: String
    let engine: String
}
