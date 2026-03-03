import Foundation
#if canImport(HealthKit)
import HealthKit
#endif

@Observable
final class HealthSensor {
    #if canImport(HealthKit)
    private let store = HKHealthStore()
    #endif
    var isAuthorized = false

    var steps: Int = 0
    var heartRate: Double = 0
    var sleepHours: Double = 0

    struct LifeEvent {
        let type: String
        let source: String
        let timestamp: Date
        let data: [String: String]
    }

    func requestAuthorization() async -> Bool {
        #if canImport(HealthKit)
        guard HKHealthStore.isHealthDataAvailable() else { return false }

        let readTypes: Set<HKObjectType> = [
            HKQuantityType(.stepCount),
            HKQuantityType(.heartRate),
            HKCategoryType(.sleepAnalysis),
        ]

        do {
            try await store.requestAuthorization(toShare: [], read: readTypes)
            isAuthorized = true
            return true
        } catch {
            print("HealthKit auth failed: \(error)")
            return false
        }
        #else
        return false
        #endif
    }

    func fetchTodaySteps() async -> Int {
        #if canImport(HealthKit)
        let type = HKQuantityType(.stepCount)
        let start = Calendar.current.startOfDay(for: Date())
        let predicate = HKQuery.predicateForSamples(withStart: start, end: Date())

        do {
            let result = try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Int, Error>) in
                let query = HKStatisticsQuery(quantityType: type, quantitySamplePredicate: predicate, options: .cumulativeSum) { _, stats, error in
                    if let error {
                        continuation.resume(throwing: error)
                        return
                    }
                    let count = stats?.sumQuantity()?.doubleValue(for: .count()) ?? 0
                    continuation.resume(returning: Int(count))
                }
                store.execute(query)
            }
            steps = result
            return result
        } catch {
            print("Steps fetch failed: \(error)")
            return 0
        }
        #else
        return 0
        #endif
    }

    func fetchLatestHeartRate() async -> Double {
        #if canImport(HealthKit)
        let type = HKQuantityType(.heartRate)
        let sort = NSSortDescriptor(key: HKSampleSortIdentifierStartDate, ascending: false)

        do {
            let result = try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Double, Error>) in
                let query = HKSampleQuery(sampleType: type, predicate: nil, limit: 1, sortDescriptors: [sort]) { _, samples, error in
                    if let error {
                        continuation.resume(throwing: error)
                        return
                    }
                    let bpm = (samples?.first as? HKQuantitySample)?
                        .quantity.doubleValue(for: HKUnit.count().unitDivided(by: .minute())) ?? 0
                    continuation.resume(returning: bpm)
                }
                store.execute(query)
            }
            heartRate = result
            return result
        } catch {
            print("Heart rate fetch failed: \(error)")
            return 0
        }
        #else
        return 0
        #endif
    }

    func fetchSleepHours() async -> Double {
        #if canImport(HealthKit)
        let type = HKCategoryType(.sleepAnalysis)
        let start = Calendar.current.date(byAdding: .day, value: -1, to: Date())!
        let predicate = HKQuery.predicateForSamples(withStart: start, end: Date())

        do {
            let result = try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Double, Error>) in
                let query = HKSampleQuery(sampleType: type, predicate: predicate, limit: HKObjectQueryNoLimit, sortDescriptors: nil) { _, samples, error in
                    if let error {
                        continuation.resume(throwing: error)
                        return
                    }
                    let totalSeconds = (samples ?? []).reduce(0.0) { total, sample in
                        total + sample.endDate.timeIntervalSince(sample.startDate)
                    }
                    continuation.resume(returning: totalSeconds / 3600.0)
                }
                store.execute(query)
            }
            sleepHours = result
            return result
        } catch {
            print("Sleep fetch failed: \(error)")
            return 0
        }
        #else
        return 0
        #endif
    }

    func toLifeEvents() -> [LifeEvent] {
        var events: [LifeEvent] = []

        if steps > 0 {
            events.append(LifeEvent(
                type: "health.steps",
                source: "healthkit",
                timestamp: Date(),
                data: ["count": "\(steps)"]
            ))
        }
        if heartRate > 0 {
            events.append(LifeEvent(
                type: "health.heart_rate",
                source: "healthkit",
                timestamp: Date(),
                data: ["bpm": String(format: "%.0f", heartRate)]
            ))
        }
        if sleepHours > 0 {
            events.append(LifeEvent(
                type: "health.sleep",
                source: "healthkit",
                timestamp: Date(),
                data: ["hours": String(format: "%.1f", sleepHours)]
            ))
        }

        return events
    }
}
