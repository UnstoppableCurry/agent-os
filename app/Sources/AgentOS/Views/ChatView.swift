import SwiftUI

public struct ChatView: View {
    let botId: String
    let botName: String

    @State private var lines: [TermLine] = []
    @State private var inputText = ""
    @State private var webSocketTask: URLSessionWebSocketTask?
    @State private var isConnected = false

    public init(botId: String, botName: String) {
        self.botId = botId
        self.botName = botName
    }

    public var body: some View {
        VStack(spacing: 0) {
            // ─── 终端输出区域 ───
            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 0) {
                        ForEach(lines) { line in
                            Text(line.text)
                                .font(.system(size: 13, design: .monospaced))
                                .foregroundStyle(.white)
                                .textSelection(.enabled)
                                .id(line.id)
                        }
                    }
                    .padding(8)
                    .frame(maxWidth: .infinity, alignment: .leading)
                }
                .background(Color(white: 0.1))
                .onChange(of: lines.count) {
                    if let last = lines.last {
                        withAnimation(.easeOut(duration: 0.1)) {
                            proxy.scrollTo(last.id, anchor: .bottom)
                        }
                    }
                }
            }

            Divider()

            // ─── 输入栏 ───
            HStack(spacing: 8) {
                Text(">")
                    .font(.system(size: 13, design: .monospaced))
                    .foregroundStyle(.green)
                TextField("输入消息...", text: $inputText)
                    .font(.system(size: 13, design: .monospaced))
                    .textFieldStyle(.plain)
                    .onSubmit { sendInput() }
                Button {
                    sendInput()
                } label: {
                    Image(systemName: "paperplane.fill")
                        .foregroundStyle(inputText.isEmpty ? .gray : .green)
                }
                .buttonStyle(.plain)
                .disabled(inputText.isEmpty)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            .background(Color(white: 0.1))
        }
        .navigationTitle(botName)
        .onAppear { connectWebSocket() }
        .onDisappear { disconnectWebSocket() }
    }

    // MARK: - WebSocket

    private func connectWebSocket() {
        let serverURL = UserDefaults.standard.string(forKey: "serverURL") ?? "http://127.0.0.1:3000"
        let wsURL = serverURL
            .replacingOccurrences(of: "http://", with: "ws://")
            .replacingOccurrences(of: "https://", with: "wss://")

        guard let url = URL(string: "\(wsURL)/v1/bots/\(botId)/ws") else {
            appendLine("[错误] 无效的 WebSocket URL")
            return
        }

        let session = URLSession(configuration: .default)
        let task = session.webSocketTask(with: url)
        task.resume()
        webSocketTask = task
        isConnected = true

        appendLine("✅ 已连接到 \(botName)")
        appendLine("输入消息开始对话\n")
        receiveLoop()
    }

    private func receiveLoop() {
        webSocketTask?.receive { result in
            switch result {
            case .success(let message):
                Task { @MainActor in
                    switch message {
                    case .string(let text):
                        // Backend sends plain text — display directly
                        appendLine(text)
                    case .data(let data):
                        if let text = String(data: data, encoding: .utf8) {
                            appendLine(text)
                        }
                    @unknown default:
                        break
                    }
                    receiveLoop()
                }
            case .failure(let error):
                Task { @MainActor in
                    if isConnected {
                        appendLine("[断开连接] \(error.localizedDescription)")
                        isConnected = false
                    }
                }
            }
        }
    }

    private func sendInput() {
        let text = inputText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty else { return }
        inputText = ""

        // Echo user input
        appendLine("> \(text)")

        let wsMessage = URLSessionWebSocketTask.Message.string(text)
        webSocketTask?.send(wsMessage) { error in
            if let error {
                Task { @MainActor in
                    appendLine("[发送失败] \(error.localizedDescription)")
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
    private func appendLine(_ text: String) {
        lines.append(TermLine(text: text))
    }
}

struct TermLine: Identifiable {
    let id = UUID()
    let text: String
}
