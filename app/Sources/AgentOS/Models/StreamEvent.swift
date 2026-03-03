import Foundation

enum StreamEvent: Codable {
    case messageStart(messageId: String)
    case contentDelta(text: String)
    case thinking(text: String)
    case toolUse(id: String, name: String, input: String)
    case toolResult(toolUseId: String, content: String)
    case messageStop(stopReason: String)
    case error(message: String)

    enum CodingKeys: String, CodingKey {
        case type, messageId, text, id, name, input, toolUseId, content, stopReason, message
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(String.self, forKey: .type)

        switch type {
        case "message_start":
            let messageId = try container.decode(String.self, forKey: .messageId)
            self = .messageStart(messageId: messageId)
        case "content_delta":
            let text = try container.decode(String.self, forKey: .text)
            self = .contentDelta(text: text)
        case "thinking":
            let text = try container.decode(String.self, forKey: .text)
            self = .thinking(text: text)
        case "tool_use":
            let id = try container.decode(String.self, forKey: .id)
            let name = try container.decode(String.self, forKey: .name)
            let input = try container.decodeIfPresent(String.self, forKey: .input) ?? ""
            self = .toolUse(id: id, name: name, input: input)
        case "tool_result":
            let toolUseId = try container.decode(String.self, forKey: .toolUseId)
            let content = try container.decode(String.self, forKey: .content)
            self = .toolResult(toolUseId: toolUseId, content: content)
        case "message_stop":
            let stopReason = try container.decodeIfPresent(String.self, forKey: .stopReason) ?? "end_turn"
            self = .messageStop(stopReason: stopReason)
        default:
            let msg = try container.decodeIfPresent(String.self, forKey: .message) ?? "Unknown event: \(type)"
            self = .error(message: msg)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .messageStart(let messageId):
            try container.encode("message_start", forKey: .type)
            try container.encode(messageId, forKey: .messageId)
        case .contentDelta(let text):
            try container.encode("content_delta", forKey: .type)
            try container.encode(text, forKey: .text)
        case .thinking(let text):
            try container.encode("thinking", forKey: .type)
            try container.encode(text, forKey: .text)
        case .toolUse(let id, let name, let input):
            try container.encode("tool_use", forKey: .type)
            try container.encode(id, forKey: .id)
            try container.encode(name, forKey: .name)
            try container.encode(input, forKey: .input)
        case .toolResult(let toolUseId, let content):
            try container.encode("tool_result", forKey: .type)
            try container.encode(toolUseId, forKey: .toolUseId)
            try container.encode(content, forKey: .content)
        case .messageStop(let stopReason):
            try container.encode("message_stop", forKey: .type)
            try container.encode(stopReason, forKey: .stopReason)
        case .error(let message):
            try container.encode("error", forKey: .type)
            try container.encode(message, forKey: .message)
        }
    }
}
