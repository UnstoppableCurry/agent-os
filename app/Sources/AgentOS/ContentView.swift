import SwiftUI

public struct ContentView: View {
    @State private var selectedTab = 0

    public init() {}

    public var body: some View {
        TabView(selection: $selectedTab) {
            BotListView()
                .tabItem {
                    Label("机器人", systemImage: "cpu")
                }
                .tag(0)

            DashboardView()
                .tabItem {
                    Label("仪表盘", systemImage: "chart.bar")
                }
                .tag(1)

            SettingsView()
                .tabItem {
                    Label("设置", systemImage: "gear")
                }
                .tag(2)
        }
    }
}
