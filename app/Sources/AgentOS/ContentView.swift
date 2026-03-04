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
            sidebar
        } detail: {
            detailView
        }
        .navigationSplitViewStyle(.balanced)
    }

    // MARK: - Sidebar

    private var sidebar: some View {
        VStack(spacing: 0) {
            List(selection: $selectedBotId) {
                if bots.isEmpty && !isLoading {
                    VStack(spacing: 12) {
                        Image(systemName: "cpu")
                            .font(.system(size: 40))
                            .foregroundStyle(.tertiary)
                        Text("暂无机器人")
                            .foregroundStyle(.secondary)
                        Text("点击下方 + 创建")
                            .font(.caption)
                            .foregroundStyle(.tertiary)
                    }
                    .frame(maxWidth: .infinity)
                    .padding(.top, 40)
                    .listRowBackground(Color.clear)
                    .listRowSeparator(.hidden)
                } else {
                    ForEach(bots, id: \.id) { bot in
                        BotSidebarRow(bot: bot, isSelected: selectedBotId == bot.id)
                            .tag(bot.id)
                    }
                    .onDelete { indexSet in
                        Task {
                            for idx in indexSet {
                                let bot = bots[idx]
                                _ = await APIClient.shared.stopBot(id: bot.id)
                            }
                            bots.remove(atOffsets: indexSet)
                            if let selected = selectedBotId,
                               !bots.contains(where: { $0.id == selected }) {
                                selectedBotId = nil
                            }
                        }
                    }
                }
            }
            .listStyle(.sidebar)

            Divider()

            // Bottom toolbar
            HStack(spacing: 16) {
                Button {
                    showCreateSheet = true
                } label: {
                    Image(systemName: "plus.circle.fill")
                        .font(.title2)
                        .foregroundStyle(.blue)
                }
                .buttonStyle(.plain)
                .help("新建机器人")

                Button {
                    Task { await loadBots() }
                } label: {
                    Image(systemName: "arrow.clockwise")
                        .font(.body)
                        .foregroundStyle(.secondary)
                }
                .buttonStyle(.plain)
                .help("刷新列表")

                Spacer()

                Button {
                    showSettings = true
                } label: {
                    Image(systemName: "gearshape")
                        .font(.body)
                        .foregroundStyle(.secondary)
                }
                .buttonStyle(.plain)
                .help("设置")
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 10)
        }
        .navigationTitle("AgentOS")
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
                            Button("完成") { showSettings = false }
                        }
                    }
            }
            .frame(minWidth: 400, minHeight: 300)
        }
        .task { await loadBots() }
    }

    // MARK: - Detail

    @ViewBuilder
    private var detailView: some View {
        if let botId = selectedBotId,
           let bot = bots.first(where: { $0.id == botId }) {
            ChatView(botId: bot.id, botName: bot.name, botEngine: bot.engine)
                .id(bot.id)
        } else {
            VStack(spacing: 16) {
                Image(systemName: "message.badge.waveform")
                    .font(.system(size: 56))
                    .foregroundStyle(.quaternary)
                Text("选择一个机器人开始对话")
                    .font(.title3)
                    .foregroundStyle(.secondary)
                Text("从左侧列表选择，或点击 + 创建新机器人")
                    .font(.caption)
                    .foregroundStyle(.tertiary)
            }
        }
    }

    private func loadBots() async {
        isLoading = true
        bots = await APIClient.shared.listBots()
        isLoading = false
    }
}

// MARK: - Sidebar Bot Row

struct BotSidebarRow: View {
    let bot: BotResponse
    let isSelected: Bool

    var body: some View {
        HStack(spacing: 10) {
            ZStack {
                RoundedRectangle(cornerRadius: 8)
                    .fill(engineColor.opacity(0.15))
                    .frame(width: 36, height: 36)
                Text(engineEmoji)
                    .font(.title3)
            }

            VStack(alignment: .leading, spacing: 2) {
                Text(bot.name)
                    .font(.body.weight(.medium))
                    .lineLimit(1)
                HStack(spacing: 4) {
                    Circle()
                        .fill(stateColor)
                        .frame(width: 6, height: 6)
                    Text(stateLabel)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    Text("·")
                        .foregroundStyle(.tertiary)
                    Text(bot.engine)
                        .font(.caption)
                        .foregroundStyle(.tertiary)
                    if let count = bot.message_count, count > 0 {
                        Text("·")
                            .foregroundStyle(.tertiary)
                        Text("\(count)条")
                            .font(.caption2)
                            .foregroundStyle(.tertiary)
                    }
                }
            }

            Spacer()

            // State icon for suspended/error
            if bot.state == "suspended" {
                Image(systemName: "pause.circle")
                    .font(.caption)
                    .foregroundStyle(.gray)
            } else if bot.state == "error" {
                Image(systemName: "exclamationmark.triangle")
                    .font(.caption)
                    .foregroundStyle(.red)
            }
        }
        .padding(.vertical, 4)
    }

    private var stateColor: Color {
        switch bot.state {
        case "running": return .green
        case "suspended": return .gray
        case "error": return .red
        case "starting": return .orange
        default: return .gray
        }
    }

    private var stateLabel: String {
        switch bot.state {
        case "running": return "运行中"
        case "suspended": return "已暂停"
        case "error": return "错误"
        case "starting": return "启动中"
        case "stopped": return "已停止"
        default: return "未知"
        }
    }

    private var engineColor: Color {
        switch bot.engine {
        case "claude": return .orange
        case "kimi": return .blue
        case "codex": return .green
        default: return .gray
        }
    }

    private var engineEmoji: String {
        switch bot.engine {
        case "claude": return "🤖"
        case "kimi": return "🌙"
        case "codex": return "💻"
        default: return "⚙️"
        }
    }
}

// MARK: - Create Bot Sheet

struct CreateBotSheet: View {
    @Environment(\.dismiss) private var dismiss
    @State private var name = ""
    @State private var engine = "claude"
    @State private var permissionMode = "bypass_permissions"
    @State private var idleTimeoutMins: Int = 30
    @State private var isCreating = false
    @State private var errorMsg: String?
    let onCreate: (BotResponse) -> Void

    var body: some View {
        NavigationStack {
            Form {
                Section("基本信息") {
                    TextField("机器人名称", text: $name)
                        .textFieldStyle(.plain)
                    Picker("引擎", selection: $engine) {
                        Label("Claude Code", systemImage: "cpu").tag("claude")
                        Label("Kimi", systemImage: "moon.stars").tag("kimi")
                        Label("Codex", systemImage: "terminal").tag("codex")
                    }
                }

                Section("权限模式") {
                    Picker("权限", selection: $permissionMode) {
                        Text("跳过所有权限").tag("bypass_permissions")
                        Text("自动接受编辑").tag("accept_edits")
                        Text("计划模式").tag("plan")
                        Text("默认 (需确认)").tag("default")
                    }
                    .pickerStyle(.menu)
                }

                Section("高级") {
                    Stepper("空闲超时: \(idleTimeoutMins) 分钟", value: $idleTimeoutMins, in: 5...120, step: 5)
                }

                if let errorMsg {
                    Section {
                        Label(errorMsg, systemImage: "exclamationmark.triangle")
                            .foregroundStyle(.red)
                            .font(.caption)
                    }
                }
            }
            .formStyle(.grouped)
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
        .frame(minWidth: 350, minHeight: 280)
    }

    private func createBot() async {
        isCreating = true
        errorMsg = nil
        if let bot = await APIClient.shared.createBot(
            name: name,
            engine: engine,
            permissionMode: permissionMode,
            idleTimeoutMins: idleTimeoutMins
        ) {
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

extension BotResponse: Identifiable {}
