import Foundation

public enum MessageRole: String, Codable, Sendable {
    case user
    case assistant
    case system
}

public enum MessageType: String, Codable, Sendable {
    case text
    case thinking
    case toolUse = "tool_use"
    case toolResult = "tool_result"
}

public struct Message: Identifiable, Codable, Sendable {
    public let id: UUID
    public let role: MessageRole
    public var content: String
    public let type: MessageType
    public let timestamp: Date
    public var toolName: String?

    public init(role: MessageRole, content: String, type: MessageType = .text, toolName: String? = nil) {
        self.id = UUID()
        self.role = role
        self.content = content
        self.type = type
        self.timestamp = Date()
        self.toolName = toolName
    }
}
