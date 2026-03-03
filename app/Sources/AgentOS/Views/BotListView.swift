import SwiftUI

public struct BotListView: View {
    @State private var bots: [Bot] = [
        Bot(name: "General Assistant", engine: .claude),
        Bot(name: "Code Helper", engine: .kimi),
    ]
    @State private var showCreateSheet = false

    public init() {}

    public var body: some View {
        NavigationStack {
            List {
                ForEach(bots) { bot in
                    BotRow(bot: bot, onToggle: { toggleBot(bot) })
                }
                .onDelete(perform: deleteBots)
            }
            .navigationTitle("机器人")
            .toolbar {
                Button {
                    showCreateSheet = true
                } label: {
                    Image(systemName: "plus")
                }
            }
            .sheet(isPresented: $showCreateSheet) {
                CreateBotSheet { newBot in
                    bots.append(newBot)
                }
            }
        }
    }

    private func toggleBot(_ bot: Bot) {
        if let idx = bots.firstIndex(where: { $0.id == bot.id }) {
            bots[idx].status = bots[idx].status == .running ? .stopped : .running
        }
    }

    private func deleteBots(at offsets: IndexSet) {
        bots.remove(atOffsets: offsets)
    }
}

struct BotRow: View {
    let bot: Bot
    let onToggle: () -> Void

    var body: some View {
        HStack(spacing: 12) {
            Circle()
                .fill(statusColor)
                .frame(width: 10, height: 10)
            VStack(alignment: .leading, spacing: 2) {
                Text(bot.name)
                    .font(.headline)
                Text(bot.engine.rawValue.capitalized)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Spacer()
            Button(bot.status == .running ? "停止" : "启动") {
                onToggle()
            }
            .buttonStyle(.bordered)
            .tint(bot.status == .running ? .red : .green)
            .controlSize(.small)
        }
        .padding(.vertical, 4)
    }

    private var statusColor: Color {
        switch bot.status {
        case .running: return .green
        case .stopped: return .gray
        case .error: return .red
        }
    }
}

struct CreateBotSheet: View {
    @Environment(\.dismiss) private var dismiss
    @State private var name = ""
    @State private var engine: BotEngine = .claude
    let onCreate: (Bot) -> Void

    var body: some View {
        NavigationStack {
            Form {
                TextField("机器人名称", text: $name)
                Picker("引擎", selection: $engine) {
                    ForEach(BotEngine.allCases, id: \.self) { e in
                        Text(e.rawValue.capitalized).tag(e)
                    }
                }
            }
            .navigationTitle("新建机器人")
            #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("取消") { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button("创建") {
                        guard !name.isEmpty else { return }
                        onCreate(Bot(name: name, engine: engine))
                        dismiss()
                    }
                    .disabled(name.isEmpty)
                }
            }
        }
        .presentationDetents([.medium])
    }
}
