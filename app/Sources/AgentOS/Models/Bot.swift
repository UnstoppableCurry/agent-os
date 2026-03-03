import Foundation

enum BotEngine: String, Codable, CaseIterable {
    case claude
    case kimi
    case gemini
}

enum BotStatus: String, Codable {
    case running
    case stopped
    case error
}

@Observable
final class Bot: Identifiable, Codable {
    let id: UUID
    var name: String
    var engine: BotEngine
    var status: BotStatus
    let createdAt: Date

    init(name: String, engine: BotEngine) {
        self.id = UUID()
        self.name = name
        self.engine = engine
        self.status = .stopped
        self.createdAt = Date()
    }

    var statusColor: String {
        switch status {
        case .running: return "green"
        case .stopped: return "gray"
        case .error: return "red"
        }
    }
}
