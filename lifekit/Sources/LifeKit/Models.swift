import Foundation

// MARK: - AnyCodable

/// Wrapper for arbitrary Codable values.
public struct AnyCodable: Codable, Sendable {
    public let value: Any

    public init(_ value: Any) {
        self.value = value
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if container.decodeNil() {
            value = NSNull()
        } else if let bool = try? container.decode(Bool.self) {
            value = bool
        } else if let int = try? container.decode(Int.self) {
            value = int
        } else if let double = try? container.decode(Double.self) {
            value = double
        } else if let string = try? container.decode(String.self) {
            value = string
        } else if let array = try? container.decode([AnyCodable].self) {
            value = array.map { $0.value }
        } else if let dict = try? container.decode([String: AnyCodable].self) {
            value = dict.mapValues { $0.value }
        } else {
            throw DecodingError.dataCorruptedError(in: container, debugDescription: "Unsupported type")
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch value {
        case is NSNull:
            try container.encodeNil()
        case let bool as Bool:
            try container.encode(bool)
        case let int as Int:
            try container.encode(int)
        case let double as Double:
            try container.encode(double)
        case let string as String:
            try container.encode(string)
        case let array as [Any]:
            try container.encode(array.map { AnyCodable($0) })
        case let dict as [String: Any]:
            try container.encode(dict.mapValues { AnyCodable($0) })
        default:
            throw EncodingError.invalidValue(value, .init(codingPath: encoder.codingPath, debugDescription: "Unsupported type: \(type(of: value))"))
        }
    }
}

// MARK: - LifeEvent

/// An event captured from an app to be sent to the AgentOS memory stream.
public struct LifeEvent: Codable, Sendable {
    public let ts: Date
    public let source: String
    public let type: String
    public let data: [String: AnyCodable]
    public let meta: [String: AnyCodable]?

    public init(
        ts: Date = Date(),
        source: String,
        type: String,
        data: [String: AnyCodable],
        meta: [String: AnyCodable]? = nil
    ) {
        self.ts = ts
        self.source = source
        self.type = type
        self.data = data
        self.meta = meta
    }
}

// MARK: - LifeInsight

/// An insight pushed from AgentOS back to the app.
public struct LifeInsight: Codable, Sendable {
    public let type: String
    public let content: String
    public let confidence: Double
    public let timestamp: Date

    public init(type: String, content: String, confidence: Double, timestamp: Date = Date()) {
        self.type = type
        self.content = content
        self.confidence = confidence
        self.timestamp = timestamp
    }
}

// MARK: - LifeKitConfig

/// Configuration for the LifeKit SDK.
public struct LifeKitConfig: Sendable {
    public let appId: String
    public let serverURL: String
    public var privacySettings: PrivacySettings

    public init(appId: String, serverURL: String, privacySettings: PrivacySettings = PrivacySettings()) {
        self.appId = appId
        self.serverURL = serverURL
        self.privacySettings = privacySettings
    }
}
