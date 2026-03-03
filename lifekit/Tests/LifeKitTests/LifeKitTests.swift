import XCTest
@testable import LifeKit

final class LifeKitTests: XCTestCase {

    // MARK: - LifeEvent Encoding/Decoding

    func testLifeEventRoundTrip() throws {
        let event = LifeEvent(
            ts: Date(timeIntervalSince1970: 1000000),
            source: "test-app",
            type: "page_view",
            data: [
                "url": AnyCodable("/home"),
                "duration": AnyCodable(3.5)
            ],
            meta: [
                "device": AnyCodable("iPhone")
            ]
        )

        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .iso8601
        let data = try encoder.encode(event)

        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        let decoded = try decoder.decode(LifeEvent.self, from: data)

        XCTAssertEqual(decoded.source, "test-app")
        XCTAssertEqual(decoded.type, "page_view")
        XCTAssertEqual(decoded.data["url"]?.value as? String, "/home")
        XCTAssertEqual(decoded.data["duration"]?.value as? Double, 3.5)
        XCTAssertEqual(decoded.meta?["device"]?.value as? String, "iPhone")
    }

    func testLifeEventWithNestedData() throws {
        let event = LifeEvent(
            source: "test-app",
            type: "complex",
            data: [
                "nested": AnyCodable(["key": "value"]),
                "list": AnyCodable([1, 2, 3])
            ]
        )

        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .iso8601
        let data = try encoder.encode(event)

        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        let decoded = try decoder.decode(LifeEvent.self, from: data)

        XCTAssertEqual(decoded.type, "complex")
    }

    // MARK: - EventBuffer

    func testEventBufferAppendAndDrain() async {
        let buffer = EventBuffer(maxEvents: 5)

        let event = LifeEvent(source: "test", type: "tap", data: [:])
        await buffer.append(event)
        await buffer.append(event)

        let count = await buffer.count
        XCTAssertEqual(count, 2)

        let drained = await buffer.drain()
        XCTAssertEqual(drained.count, 2)

        let afterDrain = await buffer.count
        XCTAssertEqual(afterDrain, 0)
    }

    func testEventBufferMaxEvents() async {
        let buffer = EventBuffer(maxEvents: 3)

        for i in 0..<5 {
            let event = LifeEvent(source: "test", type: "event_\(i)", data: [:])
            await buffer.append(event)
        }

        let count = await buffer.count
        XCTAssertEqual(count, 3)

        let events = await buffer.drain()
        XCTAssertEqual(events[0].type, "event_2")
        XCTAssertEqual(events[1].type, "event_3")
        XCTAssertEqual(events[2].type, "event_4")
    }

    // MARK: - Privacy

    func testPrivacyAllowAll() {
        let privacy = PrivacySettings()
        XCTAssertTrue(privacy.isAllowed(type: "page_view"))
        XCTAssertTrue(privacy.isAllowed(type: "location"))
        XCTAssertTrue(privacy.isAllowed(type: "purchase"))
    }

    func testPrivacyBlockType() {
        var privacy = PrivacySettings()
        privacy.block(type: "location")

        XCTAssertTrue(privacy.isAllowed(type: "page_view"))
        XCTAssertFalse(privacy.isAllowed(type: "location"))
    }

    func testPrivacyUnblock() {
        var privacy = PrivacySettings(blockedTypes: ["location", "purchase"])
        XCTAssertFalse(privacy.isAllowed(type: "location"))

        privacy.allow(type: "location")
        XCTAssertTrue(privacy.isAllowed(type: "location"))
        XCTAssertFalse(privacy.isAllowed(type: "purchase"))
    }

    func testPrivacyFilter() {
        var privacy = PrivacySettings()
        privacy.block(type: "secret")

        let events = [
            LifeEvent(source: "app", type: "page_view", data: [:]),
            LifeEvent(source: "app", type: "secret", data: [:]),
            LifeEvent(source: "app", type: "tap", data: [:])
        ]

        let filtered = privacy.filter(events: events)
        XCTAssertEqual(filtered.count, 2)
        XCTAssertEqual(filtered[0].type, "page_view")
        XCTAssertEqual(filtered[1].type, "tap")
    }

    // MARK: - LifeInsight

    func testLifeInsightDecode() throws {
        let json = """
        {
            "type": "suggestion",
            "content": "You usually run at 7am",
            "confidence": 0.85,
            "timestamp": "2026-03-03T10:00:00Z"
        }
        """
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        let insight = try decoder.decode(LifeInsight.self, from: json.data(using: .utf8)!)

        XCTAssertEqual(insight.type, "suggestion")
        XCTAssertEqual(insight.content, "You usually run at 7am")
        XCTAssertEqual(insight.confidence, 0.85)
    }
}
