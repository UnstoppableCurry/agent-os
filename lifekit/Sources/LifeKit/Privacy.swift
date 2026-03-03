import Foundation

/// Controls which event types are allowed to be reported.
public struct PrivacySettings: Sendable {
    /// When empty, all event types are allowed.
    private var blockedTypes: Set<String>

    /// Create privacy settings. By default all event types are allowed.
    public init(blockedTypes: Set<String> = []) {
        self.blockedTypes = blockedTypes
    }

    /// Block a specific event type from being reported.
    public mutating func block(type: String) {
        blockedTypes.insert(type)
    }

    /// Allow a previously blocked event type.
    public mutating func allow(type: String) {
        blockedTypes.remove(type)
    }

    /// Check if an event type is allowed.
    public func isAllowed(type: String) -> Bool {
        !blockedTypes.contains(type)
    }

    /// Filter events, returning only those with allowed types.
    public func filter(events: [LifeEvent]) -> [LifeEvent] {
        events.filter { isAllowed(type: $0.type) }
    }
}
