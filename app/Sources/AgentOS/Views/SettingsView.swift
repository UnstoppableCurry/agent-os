import SwiftUI

struct SettingsView: View {
    @AppStorage("serverURL") private var serverURL = "http://localhost:3000"
    @AppStorage("healthKitEnabled") private var healthKitEnabled = false
    @AppStorage("contactsEnabled") private var contactsEnabled = false
    @AppStorage("calendarEnabled") private var calendarEnabled = false
    @AppStorage("locationEnabled") private var locationEnabled = false
    @AppStorage("screenTimeEnabled") private var screenTimeEnabled = false

    var body: some View {
        NavigationStack {
            Form {
                serverSection
                sensorSection
                privacySection
                aboutSection
            }
            .navigationTitle("Settings")
        }
    }

    private var serverSection: some View {
        Section("Server") {
            TextField("Server URL", text: $serverURL)
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()
                .keyboardType(.URL)

            Button("Test Connection") {
                Task {
                    await testConnection()
                }
            }
        }
    }

    private var sensorSection: some View {
        Section("Sensors") {
            Toggle("HealthKit", isOn: $healthKitEnabled)
            Toggle("Contacts", isOn: $contactsEnabled)
            Toggle("Calendar", isOn: $calendarEnabled)
            Toggle("Location", isOn: $locationEnabled)
            Toggle("Screen Time", isOn: $screenTimeEnabled)
        }
    }

    private var privacySection: some View {
        Section("Privacy") {
            NavigationLink("Data Collected") {
                PrivacyDetailView()
            }
            Button("Clear Local Data", role: .destructive) {
                // Clear cached data
            }
        }
    }

    private var aboutSection: some View {
        Section("About") {
            LabeledContent("Version", value: "0.1.0")
            LabeledContent("Build", value: "1")
            Link("Source Code", destination: URL(string: "https://github.com/UnstoppableCurry/agent-os")!)
        }
    }

    private func testConnection() async {
        let health = await APIClient.shared.getHealth()
        // TODO: Show result to user
    }
}

struct PrivacyDetailView: View {
    var body: some View {
        List {
            Section("What we collect") {
                Label("Step count and activity data", systemImage: "figure.walk")
                Label("Sleep duration", systemImage: "bed.double")
                Label("Heart rate summaries", systemImage: "heart")
                Label("Calendar event titles and times", systemImage: "calendar")
                Label("Contact names (no phone numbers)", systemImage: "person.2")
            }
            Section("What we never collect") {
                Label("Message content", systemImage: "xmark.circle")
                Label("Photos or media", systemImage: "xmark.circle")
                Label("Passwords or credentials", systemImage: "xmark.circle")
                Label("Precise location history", systemImage: "xmark.circle")
            }
        }
        .navigationTitle("Privacy")
    }
}

#Preview {
    SettingsView()
}
