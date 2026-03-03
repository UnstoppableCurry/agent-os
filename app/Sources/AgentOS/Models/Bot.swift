import Foundation

public enum BotEngine: String, Codable, CaseIterable, Sendable {
    case claude
    case kimi
    case gemini
}

public enum BotState: String, Codable, Sendable {
    case running
    case stopped
    case error
}

public struct Bot: Identifiable, Sendable {
    public let id: UUID
    public var name: String
    public var engine: BotEngine
    public var status: BotState
    public let createdAt: Date

    public init(name: String, engine: BotEngine) {
        self.id = UUID()
        self.name = name
        self.engine = engine
        self.status = .stopped
        self.createdAt = Date()
    }

    public var statusColor: String {
        switch status {
        case .running: return "green"
        case .stopped: return "gray"
        case .error: return "red"
        }
    }
}
