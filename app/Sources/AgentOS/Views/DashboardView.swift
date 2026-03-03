import SwiftUI

public struct DashboardView: View {
    @State private var healthData = HealthSnapshot()

    public init() {}

    public var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 16) {
                    todayOverview
                    crystalGrid
                    recentActivity
                }
                .padding()
            }
            .navigationTitle("仪表盘")
            .refreshable { await refreshData() }
        }
    }

    private var todayOverview: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("今日").font(.headline)
            HStack(spacing: 16) {
                StatCard(title: "步数", value: "\(healthData.steps)", icon: "figure.walk", color: .green)
                StatCard(title: "睡眠", value: healthData.sleepHours, icon: "bed.double", color: .indigo)
                StatCard(title: "心率", value: "\(healthData.heartRate) bpm", icon: "heart.fill", color: .red)
            }
        }
    }

    private var crystalGrid: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("水晶").font(.headline)
            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                CrystalCard(name: "健康", value: "85%", icon: "heart.circle", color: .pink)
                CrystalCard(name: "社交", value: "12 联系人", icon: "person.2", color: .blue)
                CrystalCard(name: "日历", value: "3 事件", icon: "calendar", color: .orange)
                CrystalCard(name: "屏幕", value: "4h 23m", icon: "iphone", color: .purple)
            }
        }
    }

    private var recentActivity: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("最近活动").font(.headline)
            ForEach(sampleActivities, id: \.time) { activity in
                HStack(spacing: 12) {
                    Image(systemName: activity.icon)
                        .foregroundStyle(activity.color)
                        .frame(width: 24)
                    VStack(alignment: .leading) {
                        Text(activity.title).font(.subheadline)
                        Text(activity.time).font(.caption).foregroundStyle(.secondary)
                    }
                    Spacer()
                }
                .padding(.vertical, 2)
            }
        }
    }

    private func refreshData() async {}

    private var sampleActivities: [ActivityItem] {
        [
            ActivityItem(title: "晨间散步完成", time: "8:30 AM", icon: "figure.walk", color: .green),
            ActivityItem(title: "团队会议", time: "10:00 AM", icon: "calendar", color: .orange),
            ActivityItem(title: "检测到心率升高", time: "2:15 PM", icon: "heart.fill", color: .red),
        ]
    }
}

struct StatCard: View {
    let title: String; let value: String; let icon: String; let color: Color
    var body: some View {
        VStack(spacing: 6) {
            Image(systemName: icon).font(.title2).foregroundStyle(color)
            Text(value).font(.headline)
            Text(title).font(.caption).foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity).padding()
        .background(color.opacity(0.1))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}

struct CrystalCard: View {
    let name: String; let value: String; let icon: String; let color: Color
    var body: some View {
        HStack(spacing: 10) {
            Image(systemName: icon).font(.title3).foregroundStyle(color)
            VStack(alignment: .leading) {
                Text(name).font(.caption.bold())
                Text(value).font(.caption).foregroundStyle(.secondary)
            }
            Spacer()
        }
        .padding()
        .background(Color.gray.opacity(0.15))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}

struct ActivityItem { let title: String; let time: String; let icon: String; let color: Color }
struct HealthSnapshot { var steps: Int = 6432; var sleepHours: String = "7h 20m"; var heartRate: Int = 72 }
