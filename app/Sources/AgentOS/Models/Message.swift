import Foundation

enum MessageRole: String, Codable {
    case user
    case assistant
    case system
}

enum MessageType: String, Codable {
    case text
    case thinking
    case toolUse = "tool_use"
    case toolResult = "tool_result"
}

struct Message: Identifiable, Codable {
    let id: UUID
    let role: MessageRole
    var content: String
    let type: MessageType
    let timestamp: Date
    var toolName: String?

    init(role: MessageRole, content: String, type: MessageType = .text, toolName: String? = nil) {
        self.id = UUID()
        self.role = role
        self.content = content
        self.type = type
        self.timestamp = Date()
        self.toolName = toolName
    }
}
