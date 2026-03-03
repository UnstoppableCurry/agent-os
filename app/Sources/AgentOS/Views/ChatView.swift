import SwiftUI

public struct ChatView: View {
    let botId: String
    let botName: String

    @State private var lines: [ConsoleLine] = []
    @State private var inputText = ""
    @State private var isStreaming = false

    public init(botId: String, botName: String) {
        self.botId = botId
        self.botName = botName
    }

    public var body: some View {
        VStack(spacing: 0) {
            // Terminal output area
            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 2) {
                        ForEach(lines) { line in
                            Text(line.text)
                                .font(.system(.body, design: .monospaced))
                                .foregroundStyle(line.color)
                                .textSelection(.enabled)
                                .id(line.id)
                        }
                    }
                    .padding(12)
                    .frame(maxWidth: .infinity, alignment: .leading)
                }
                .background(Color.black)
                .onChange(of: lines.count) {
                    if let last = lines.last {
                        withAnimation {
                            proxy.scrollTo(last.id, anchor: .bottom)
                        }
                    }
                }
            }

            Divider()

            // Input bar
            HStack(spacing: 8) {
                Text(">")
                    .font(.system(.body, design: .monospaced))
                    .foregroundStyle(.green)
                TextField("输入消息...", text: $inputText)
                    .font(.system(.body, design: .monospaced))
                    .textFieldStyle(.plain)
                    .onSubmit { sendMessage() }
                    .disabled(isStreaming)
                Button {
                    sendMessage()
                } label: {
                    Image(systemName: isStreaming ? "stop.fill" : "paperplane.fill")
                        .foregroundStyle(inputText.isEmpty ? .gray : .green)
                }
                .buttonStyle(.plain)
                .disabled(inputText.isEmpty && !isStreaming)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            .background(Color.black)
        }
        .navigationTitle(botName)
        .onAppear {
            appendLine("已连接到 \(botName) (id: \(botId.prefix(8))...)", color: .gray)
            appendLine("输入消息开始对话\n", color: .gray)
        }
    }

    private func sendMessage() {
        guard !inputText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return }
        let text = inputText
        inputText = ""

        appendLine("> \(text)", color: .green)
        isStreaming = true
        // 添加空行作为回复占位
        let replyId = appendLine("", color: .white)

        Task {
            await APIClient.shared.sendMessage(text, botId: botId) { event in
                Task { @MainActor in
                    applyEvent(event, replyLineId: replyId)
                }
            }
            await MainActor.run {
                isStreaming = false
                appendLine("", color: .white) // blank line after response
            }
        }
    }

    @MainActor
    private func applyEvent(_ event: StreamEvent, replyLineId: UUID) {
        switch event {
        case .contentDelta(let text):
            if let idx = lines.firstIndex(where: { $0.id == replyLineId }) {
                lines[idx].text += text
            }
        case .thinking(let text):
            appendLine("[思考] \(text)", color: .cyan)
        case .toolUse(_, let name, let input):
            appendLine("[工具] \(name): \(input.prefix(200))", color: .orange)
        case .toolResult(_, let content):
            appendLine("[结果] \(content.prefix(300))", color: .green)
        case .messageStop:
            isStreaming = false
        case .error(let msg):
            appendLine("[错误] \(msg)", color: .red)
            isStreaming = false
        default:
            break
        }
    }

    @MainActor @discardableResult
    private func appendLine(_ text: String, color: Color) -> UUID {
        let line = ConsoleLine(text: text, color: color)
        lines.append(line)
        return line.id
    }
}

struct ConsoleLine: Identifiable {
    let id = UUID()
    var text: String
    var color: Color
}
