import SwiftUI

struct DashboardView: View {
    @State private var healthData = HealthSnapshot()

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 16) {
                    todayOverview
                    crystalGrid
                    recentActivity
                }
                .padding()
            }
            .navigationTitle("Dashboard")
            .refreshable {
                await refreshData()
            }
        }
    }

    private var todayOverview: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Today")
                .font(.headline)

            HStack(spacing: 16) {
                StatCard(title: "Steps", value: "\(healthData.steps)", icon: "figure.walk", color: .green)
                StatCard(title: "Sleep", value: healthData.sleepHours, icon: "bed.double", color: .indigo)
                StatCard(title: "Heart", value: "\(healthData.heartRate) bpm", icon: "heart.fill", color: .red)
            }
        }
    }

    private var crystalGrid: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Crystals")
                .font(.headline)

            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                CrystalCard(name: "Health", value: "85%", icon: "heart.circle", color: .pink)
                CrystalCard(name: "Social", value: "12 contacts", icon: "person.2", color: .blue)
                CrystalCard(name: "Calendar", value: "3 events", icon: "calendar", color: .orange)
                CrystalCard(name: "Screen", value: "4h 23m", icon: "iphone", color: .purple)
            }
        }
    }

    private var recentActivity: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Recent Activity")
                .font(.headline)

            ForEach(sampleActivities, id: \.time) { activity in
                HStack(spacing: 12) {
                    Image(systemName: activity.icon)
                        .foregroundStyle(activity.color)
                        .frame(width: 24)
                    VStack(alignment: .leading) {
                        Text(activity.title)
                            .font(.subheadline)
                        Text(activity.time)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                    Spacer()
                }
                .padding(.vertical, 2)
            }
        }
    }

    private func refreshData() async {
        // Will integrate with HealthSensor
    }

    private var sampleActivities: [ActivityItem] {
        [
            ActivityItem(title: "Morning walk completed", time: "8:30 AM", icon: "figure.walk", color: .green),
            ActivityItem(title: "Meeting with team", time: "10:00 AM", icon: "calendar", color: .orange),
            ActivityItem(title: "Heart rate spike detected", time: "2:15 PM", icon: "heart.fill", color: .red),
        ]
    }
}

struct StatCard: View {
    let title: String
    let value: String
    let icon: String
    let color: Color

    var body: some View {
        VStack(spacing: 6) {
            Image(systemName: icon)
                .font(.title2)
                .foregroundStyle(color)
            Text(value)
                .font(.headline)
            Text(title)
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity)
        .padding()
        .background(color.opacity(0.1))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}

struct CrystalCard: View {
    let name: String
    let value: String
    let icon: String
    let color: Color

    var body: some View {
        HStack(spacing: 10) {
            Image(systemName: icon)
                .font(.title3)
                .foregroundStyle(color)
            VStack(alignment: .leading) {
                Text(name)
                    .font(.caption.bold())
                Text(value)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Spacer()
        }
        .padding()
        .background(Color(.systemGray6))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}

struct ActivityItem {
    let title: String
    let time: String
    let icon: String
    let color: Color
}

struct HealthSnapshot {
    var steps: Int = 6432
    var sleepHours: String = "7h 20m"
    var heartRate: Int = 72
}

#Preview {
    DashboardView()
}
