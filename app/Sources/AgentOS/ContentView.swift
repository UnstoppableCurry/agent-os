import SwiftUI

public struct ContentView: View {
    @State private var bots: [BotResponse] = []
    @State private var selectedBotId: String?
    @State private var showCreateSheet = false
    @State private var showSettings = false
    @State private var isLoading = false

    public init() {}

    public var body: some View {
        NavigationSplitView {
            // ─── 左侧: Bot 列表 ───
            List(selection: $selectedBotId) {
                ForEach(bots, id: \.id) { bot in
                    HStack(spacing: 10) {
                        Circle()
                            .fill(bot.state == "running" ? Color.green : .gray)
                            .frame(width: 8, height: 8)
                        VStack(alignment: .leading, spacing: 2) {
                            Text(bot.name)
                                .font(.body)
                            Text(bot.engine)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    }
                    .tag(bot.id)
                    .padding(.vertical, 2)
                }
            }
            .navigationTitle("AgentOS")
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
                ToolbarItem(placement: .automatic) {
                    Button {
                        showSettings = true
                    } label: {
                        Image(systemName: "gear")
                    }
                }
            }
            .sheet(isPresented: $showCreateSheet) {
                CreateBotSheet { newBot in
                    bots.append(newBot)
                    selectedBotId = newBot.id
                }
            }
            .sheet(isPresented: $showSettings) {
                NavigationStack {
                    SettingsView()
                        .toolbar {
                            ToolbarItem(placement: .cancellationAction) {
                                Button("关闭") { showSettings = false }
                            }
                        }
                }
            }
            .task { await loadBots() }
        } detail: {
            // ─── 右侧: 终端窗口 ───
            if let botId = selectedBotId,
               let bot = bots.first(where: { $0.id == botId }) {
                ChatView(botId: bot.id, botName: bot.name)
                    .id(bot.id) // 切换 bot 时重建连接
            } else {
                ContentUnavailableView {
                    Label("选择一个机器人", systemImage: "message")
                } description: {
                    Text("从左侧列表选择或创建一个机器人开始对话")
                }
            }
        }
    }

    private func loadBots() async {
        isLoading = true
        bots = await APIClient.shared.listBots()
        isLoading = false
    }
}

// MARK: - 创建 Bot 面板

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
                    Text("Claude Code").tag("claude")
                    Text("Kimi").tag("kimi")
                    Text("Codex").tag("codex")
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
