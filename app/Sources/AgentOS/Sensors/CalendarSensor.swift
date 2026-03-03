import Foundation
import EventKit

@Observable
final class CalendarSensor {
    private let store = EKEventStore()
    var isAuthorized = false

    struct CalendarEvent {
        let type: String
        let source: String
        let timestamp: Date
        let data: [String: String]
    }

    func requestAuthorization() async -> Bool {
        do {
            let granted = try await store.requestFullAccessToEvents()
            isAuthorized = granted
            return granted
        } catch {
            print("Calendar auth failed: \(error)")
            return false
        }
    }

    func fetchTodayEvents() async -> [CalendarEvent] {
        guard isAuthorized else { return [] }

        let start = Calendar.current.startOfDay(for: Date())
        let end = Calendar.current.date(byAdding: .day, value: 1, to: start)!
        let predicate = store.predicateForEvents(withStart: start, end: end, calendars: nil)
        let events = store.events(matching: predicate)

        return events.map { event in
            let formatter = DateFormatter()
            formatter.timeStyle = .short

            return CalendarEvent(
                type: "calendar.event",
                source: "eventkit",
                timestamp: event.startDate,
                data: [
                    "title": event.title ?? "Untitled",
                    "start": formatter.string(from: event.startDate),
                    "end": formatter.string(from: event.endDate),
                    "location": event.location ?? "",
                    "isAllDay": event.isAllDay ? "true" : "false",
                ]
            )
        }
    }

    func todayEventCount() async -> Int {
        let events = await fetchTodayEvents()
        return events.count
    }
}
