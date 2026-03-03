import SwiftUI

public struct SettingsView: View {
    @AppStorage("serverURL") private var serverURL = "http://localhost:3000"
    @AppStorage("healthKitEnabled") private var healthKitEnabled = false
    @AppStorage("contactsEnabled") private var contactsEnabled = false
    @AppStorage("calendarEnabled") private var calendarEnabled = false
    @AppStorage("locationEnabled") private var locationEnabled = false
    @AppStorage("screenTimeEnabled") private var screenTimeEnabled = false

    public init() {}

    public var body: some View {
        NavigationStack {
            Form {
                serverSection
                sensorSection
                privacySection
                aboutSection
            }
            .navigationTitle("设置")
        }
    }

    private var serverSection: some View {
        Section("服务器") {
            TextField("服务器地址", text: $serverURL)
                #if os(iOS)
                .textInputAutocapitalization(.never)
                .keyboardType(.URL)
                #endif
                .autocorrectionDisabled()

            Button("测试连接") {
                Task { await testConnection() }
            }
        }
    }

    private var sensorSection: some View {
        Section("传感器") {
            Toggle("健康数据", isOn: $healthKitEnabled)
            Toggle("通讯录", isOn: $contactsEnabled)
            Toggle("日历", isOn: $calendarEnabled)
            Toggle("位置", isOn: $locationEnabled)
            Toggle("屏幕使用", isOn: $screenTimeEnabled)
        }
    }

    private var privacySection: some View {
        Section("隐私") {
            NavigationLink("数据收集说明") {
                PrivacyDetailView()
            }
            Button("清除本地数据", role: .destructive) {}
        }
    }

    private var aboutSection: some View {
        Section("关于") {
            LabeledContent("版本", value: "0.1.0")
            LabeledContent("构建号", value: "1")
            Link("源代码", destination: URL(string: "https://github.com/UnstoppableCurry/agent-os")!)
        }
    }

    private func testConnection() async {
        let _ = await APIClient.shared.getHealth()
    }
}

struct PrivacyDetailView: View {
    var body: some View {
        List {
            Section("我们收集的数据") {
                Label("步数和运动数据", systemImage: "figure.walk")
                Label("睡眠时长", systemImage: "bed.double")
                Label("心率摘要", systemImage: "heart")
                Label("日历事件标题和时间", systemImage: "calendar")
                Label("联系人姓名（不含电话号码）", systemImage: "person.2")
            }
            Section("我们绝不收集") {
                Label("消息内容", systemImage: "xmark.circle")
                Label("照片或媒体", systemImage: "xmark.circle")
                Label("密码或凭据", systemImage: "xmark.circle")
                Label("精确位置历史", systemImage: "xmark.circle")
            }
        }
        .navigationTitle("隐私")
    }
}
