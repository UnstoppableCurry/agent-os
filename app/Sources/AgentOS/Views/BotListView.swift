import SwiftUI

public struct BotListView: View {
    @State private var bots: [BotResponse] = []
    @State private var showCreateSheet = false
    @State private var isLoading = false
    @State private var selectedBot: BotResponse?

    public init() {}

    public var body: some View {
        NavigationStack {
            Group {
                if bots.isEmpty && !isLoading {
                    ContentUnavailableView {
                        Label("还没有机器人", systemImage: "cpu")
                    } description: {
                        Text("点击右上角 + 创建一个 Claude 机器人")
                    }
                } else {
                    List {
                        ForEach(bots, id: \.id) { bot in
                            BotRow(bot: bot) {
                                selectedBot = bot
                            }
                        }
                    }
                }
            }
            .navigationTitle("机器人")
            .toolbar {
                ToolbarItem(placement: .automatic) {
                    Button {
                        showCreateSheet = true
                    } label: {
                        Image(systemName: "plus")
                    }
                }
                ToolbarItem(placement: .automatic) {
                    Button {
                        Task { await loadBots() }
                    } label: {
                        Image(systemName: "arrow.clockwise")
                    }
                }
            }
            .sheet(isPresented: $showCreateSheet) {
                CreateBotSheet { newBot in
                    bots.append(newBot)
                    selectedBot = newBot
                }
            }
            .sheet(item: $selectedBot) { bot in
                NavigationStack {
                    ChatView(botId: bot.id, botName: bot.name)
                        .toolbar {
                            ToolbarItem(placement: .cancellationAction) {
                                Button("关闭") { selectedBot = nil }
                            }
                        }
                }
            }
            .task { await loadBots() }
        }
    }

    private func loadBots() async {
        isLoading = true
        bots = await APIClient.shared.listBots()
        isLoading = false
    }
}

struct BotRow: View {
    let bot: BotResponse
    let onChat: () -> Void

    var body: some View {
        HStack(spacing: 12) {
            Circle()
                .fill(bot.state == "running" ? Color.green : .gray)
                .frame(width: 10, height: 10)
            VStack(alignment: .leading, spacing: 2) {
                Text(bot.name)
                    .font(.headline)
                Text(bot.engine.capitalized)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Spacer()
            Button("对话") {
                onChat()
            }
            .buttonStyle(.bordered)
            .tint(.blue)
            .controlSize(.small)
        }
        .padding(.vertical, 4)
    }
}

struct CreateBotSheet: View {
    @Environment(\.dismiss) private var dismiss
    @State private var name = ""
    @State private var engine = "claude"
    @State private var isCreating = false
    @State private var errorMsg: String?
    let onCreate: (BotResponse) -> Void

    var body: some View {
        NavigationStack {
            Form {
                TextField("机器人名称", text: $name)
                Picker("引擎", selection: $engine) {
                    Text("Claude").tag("claude")
                    Text("Kimi").tag("kimi")
                    Text("Gemini").tag("gemini")
                }

                if let errorMsg {
                    Text(errorMsg)
                        .foregroundStyle(.red)
                        .font(.caption)
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
                        Task { await createBot() }
                    }
                    .disabled(name.isEmpty || isCreating)
                }
            }
        }
        .presentationDetents([.medium])
    }

    private func createBot() async {
        isCreating = true
        errorMsg = nil
        if let bot = await APIClient.shared.createBot(name: name, engine: engine) {
            if bot.state == "error" {
                errorMsg = "机器人启动失败，请检查后端日志"
                isCreating = false
                return
            }
            onCreate(bot)
            dismiss()
        } else {
            errorMsg = "创建失败，请检查服务器连接"
        }
        isCreating = false
    }
}

extension BotResponse: @retroactive Identifiable {}
