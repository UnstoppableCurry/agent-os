import SwiftUI

public struct ChatView: View {
    let botId: String
    let botName: String
    let botEngine: String

    @State private var lines: [TermLine] = []
    @State private var inputText = ""
    @State private var webSocketTask: URLSessionWebSocketTask?
    @State private var isConnected = false
    @State private var isWaiting = false

    public init(botId: String, botName: String, botEngine: String = "claude") {
        self.botId = botId
        self.botName = botName
        self.botEngine = botEngine
    }

    public var body: some View {
        VStack(spacing: 0) {
            // ─── 顶部状态栏 ───
            HStack(spacing: 8) {
                Circle()
                    .fill(isConnected ? Color.green : .red)
                    .frame(width: 8, height: 8)
                Text(botName)
                    .font(.headline)
                Text("·")
                    .foregroundStyle(.tertiary)
                Text(botEngine)
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                Spacer()
                if isWaiting {
                    ProgressView()
                        .controlSize(.small)
                    Text("处理中...")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Button {
                    reconnect()
                } label: {
                    Image(systemName: "arrow.clockwise")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                .buttonStyle(.plain)
                .help("重新连接")
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 8)
            .background(.bar)

            Divider()

            // ─── 终端输出区域 ───
            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 1) {
                        ForEach(lines) { line in
                            lineView(line)
                                .id(line.id)
                        }
                    }
                    .padding(12)
                    .frame(maxWidth: .infinity, alignment: .leading)
                }
                .background(Color(nsColor: .init(white: 0.08, alpha: 1)))
                .onChange(of: lines.count) {
                    if let last = lines.last {
                        withAnimation(.easeOut(duration: 0.1)) {
                            proxy.scrollTo(last.id, anchor: .bottom)
                        }
                    }
                }
            }

            // ─── 输入栏 ───
            HStack(spacing: 10) {
                Image(systemName: "chevron.right")
                    .font(.system(size: 12, weight: .bold, design: .monospaced))
                    .foregroundStyle(.green)

                TextField("输入消息...", text: $inputText)
                    .font(.system(size: 13, design: .monospaced))
                    .textFieldStyle(.plain)
                    .onSubmit { sendInput() }
                    .disabled(!isConnected)

                Button {
                    sendInput()
                } label: {
                    Image(systemName: "paperplane.fill")
                        .font(.system(size: 14))
                        .foregroundStyle(canSend ? .blue : .gray)
                }
                .buttonStyle(.plain)
                .disabled(!canSend)
                .keyboardShortcut(.return, modifiers: .command)
            }
            .padding(.horizontal, 14)
            .padding(.vertical, 10)
            .background(Color(nsColor: .init(white: 0.12, alpha: 1)))
        }
        .onAppear { connectWebSocket() }
        .onDisappear { disconnectWebSocket() }
    }

    private var canSend: Bool {
        isConnected && !inputText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }

    // MARK: - Line Rendering

    @ViewBuilder
    private func lineView(_ line: TermLine) -> some View {
        switch line.style {
        case .system:
            Text(line.text)
                .font(.system(size: 12, design: .monospaced))
                .foregroundStyle(.gray)
                .padding(.vertical, 2)
        case .userInput:
            HStack(spacing: 6) {
                Text(">")
                    .foregroundStyle(.green)
                Text(line.text)
                    .foregroundStyle(.green)
            }
            .font(.system(size: 13, weight: .medium, design: .monospaced))
            .padding(.vertical, 3)
        case .response:
            Text(line.text)
                .font(.system(size: 13, design: .monospaced))
                .foregroundStyle(.white)
                .textSelection(.enabled)
        case .thinking:
            HStack(spacing: 6) {
                Image(systemName: "brain")
                    .font(.system(size: 10))
                    .foregroundStyle(.cyan)
                Text(line.text)
                    .font(.system(size: 12, design: .monospaced))
                    .foregroundStyle(.cyan.opacity(0.8))
            }
            .padding(.vertical, 1)
        case .tool:
            HStack(spacing: 6) {
                Image(systemName: "wrench.and.screwdriver")
                    .font(.system(size: 10))
                    .foregroundStyle(.orange)
                Text(line.text)
                    .font(.system(size: 12, weight: .medium, design: .monospaced))
                    .foregroundStyle(.orange)
            }
            .padding(.vertical, 2)
        case .toolResult:
            Text(line.text)
                .font(.system(size: 11, design: .monospaced))
                .foregroundStyle(.white.opacity(0.6))
                .padding(.leading, 20)
                .textSelection(.enabled)
        case .error:
            HStack(spacing: 6) {
                Image(systemName: "xmark.circle.fill")
                    .font(.system(size: 10))
                    .foregroundStyle(.red)
                Text(line.text)
                    .font(.system(size: 12, design: .monospaced))
                    .foregroundStyle(.red)
            }
        case .separator:
            Divider()
                .background(Color.white.opacity(0.1))
                .padding(.vertical, 6)
        }
    }

    // MARK: - WebSocket

    private func connectWebSocket() {
        let serverURL = UserDefaults.standard.string(forKey: "serverURL") ?? "http://127.0.0.1:3000"
        let wsURL = serverURL
            .replacingOccurrences(of: "http://", with: "ws://")
            .replacingOccurrences(of: "https://", with: "wss://")

        guard let url = URL(string: "\(wsURL)/v1/bots/\(botId)/ws") else {
            addLine("无效的 WebSocket URL", style: .error)
            return
        }

        let session = URLSession(configuration: .default)
        let task = session.webSocketTask(with: url)
        task.resume()
        webSocketTask = task
        isConnected = true

        addLine("已连接到 \(botName) (\(botEngine))", style: .system)
        addLine("输入消息开始对话", style: .system)
        receiveLoop()
    }

    private func reconnect() {
        disconnectWebSocket()
        lines.removeAll()
        connectWebSocket()
    }

    private func receiveLoop() {
        webSocketTask?.receive { result in
            switch result {
            case .success(let message):
                Task { @MainActor in
                    switch message {
                    case .string(let text):
                        handleIncoming(text)
                    case .data(let data):
                        if let text = String(data: data, encoding: .utf8) {
                            handleIncoming(text)
                        }
                    @unknown default:
                        break
                    }
                    receiveLoop()
                }
            case .failure(let error):
                Task { @MainActor in
                    if isConnected {
                        addLine("连接断开: \(error.localizedDescription)", style: .error)
                        isConnected = false
                        isWaiting = false
                    }
                }
            }
        }
    }

    @MainActor
    private func handleIncoming(_ text: String) {
        // Backend sends plain text — each line might be:
        // response text, 💭 thinking, 🔧 tool, 📋 result, ───separator, etc.
        if text.hasPrefix("💭") {
            addLine(String(text.dropFirst(2)).trimmingCharacters(in: .whitespaces), style: .thinking)
        } else if text.hasPrefix("🔧") {
            addLine(String(text.dropFirst(2)).trimmingCharacters(in: .whitespaces), style: .tool)
        } else if text.hasPrefix("📋") {
            addLine(String(text.dropFirst(2)).trimmingCharacters(in: .whitespaces), style: .toolResult)
        } else if text.contains("───") {
            addLine("", style: .separator)
            isWaiting = false
        } else if text.hasPrefix("[错误]") {
            addLine(text, style: .error)
            isWaiting = false
        } else {
            // Normal response text — append to last response line or create new
            if let lastIdx = lines.indices.last, lines[lastIdx].style == .response {
                lines[lastIdx].text += text
            } else {
                addLine(text, style: .response)
            }
        }
    }

    private func sendInput() {
        let text = inputText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty, isConnected else { return }
        inputText = ""

        addLine(text, style: .userInput)
        isWaiting = true

        let wsMessage = URLSessionWebSocketTask.Message.string(text)
        webSocketTask?.send(wsMessage) { error in
            if let error {
                Task { @MainActor in
                    addLine("发送失败: \(error.localizedDescription)", style: .error)
                    isWaiting = false
                }
            }
        }
    }

    private func disconnectWebSocket() {
        isConnected = false
        webSocketTask?.cancel(with: .goingAway, reason: nil)
        webSocketTask = nil
    }

    @MainActor
    private func addLine(_ text: String, style: TermLineStyle) {
        lines.append(TermLine(text: text, style: style))
    }
}

// MARK: - Data Models

enum TermLineStyle {
    case system      // 灰色系统消息
    case userInput   // 绿色用户输入
    case response    // 白色 AI 回复
    case thinking    // 青色思考
    case tool        // 橙色工具调用
    case toolResult  // 暗色工具结果
    case error       // 红色错误
    case separator   // 分隔线
}

struct TermLine: Identifiable {
    let id = UUID()
    var text: String
    let style: TermLineStyle
}
