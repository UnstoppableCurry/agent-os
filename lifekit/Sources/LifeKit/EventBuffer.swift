import Foundation

/// Offline event buffer that caches events to disk and uploads when connected.
actor EventBuffer {
    private let fileURL: URL
    private let maxEvents: Int
    private var events: [LifeEvent] = []

    init(maxEvents: Int = 1000) {
        self.maxEvents = maxEvents
        let cacheDir = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask).first!
        self.fileURL = cacheDir.appendingPathComponent("lifekit_events.json")
        self.events = Self.loadFromDisk(fileURL: self.fileURL)
    }

    /// Add an event to the buffer.
    func append(_ event: LifeEvent) {
        events.append(event)
        if events.count > maxEvents {
            events.removeFirst(events.count - maxEvents)
        }
        saveToDisk()
    }

    /// Take all buffered events, clearing the buffer.
    func drain() -> [LifeEvent] {
        let drained = events
        events.removeAll()
        saveToDisk()
        return drained
    }

    /// Number of buffered events.
    var count: Int {
        events.count
    }

    /// Peek at buffered events without removing them.
    var bufferedEvents: [LifeEvent] {
        events
    }

    // MARK: - Persistence

    private func saveToDisk() {
        do {
            let encoder = JSONEncoder()
            encoder.dateEncodingStrategy = .iso8601
            let data = try encoder.encode(events)
            try data.write(to: fileURL, options: .atomic)
        } catch {
            // Silently fail — best effort persistence
        }
    }

    private static func loadFromDisk(fileURL: URL) -> [LifeEvent] {
        guard let data = try? Data(contentsOf: fileURL) else { return [] }
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        return (try? decoder.decode([LifeEvent].self, from: data)) ?? []
    }
}
