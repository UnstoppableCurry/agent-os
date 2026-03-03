import SwiftUI

public struct ChatView: View {
    @State private var messages: [Message] = []
    @State private var inputText = ""
    @State private var isStreaming = false

    public init() {}

    public var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                messageList
                Divider()
                inputBar
            }
            .navigationTitle("Chat")
            #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
            #endif
        }
    }

    private var messageList: some View {
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 12) {
                    ForEach(messages) { message in
                        MessageBubble(message: message)
                            .id(message.id)
                    }
                }
                .padding()
            }
            .onChange(of: messages.count) {
                if let last = messages.last {
                    withAnimation {
                        proxy.scrollTo(last.id, anchor: .bottom)
                    }
                }
            }
        }
    }

    private var inputBar: some View {
        HStack(spacing: 12) {
            TextField("Message...", text: $inputText, axis: .vertical)
                .textFieldStyle(.plain)
                .lineLimit(1...5)
                .padding(10)
                .background(Color.gray.opacity(0.15))
                .clipShape(RoundedRectangle(cornerRadius: 20))

            Button {
                sendMessage()
            } label: {
                Image(systemName: isStreaming ? "stop.circle.fill" : "arrow.up.circle.fill")
                    .font(.title2)
                    .foregroundStyle(inputText.isEmpty && !isStreaming ? .gray : .blue)
            }
            .disabled(inputText.isEmpty && !isStreaming)
        }
        .padding(.horizontal)
        .padding(.vertical, 8)
    }

    private func sendMessage() {
        guard !inputText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return }
        let text = inputText
        inputText = ""

        let userMsg = Message(role: .user, content: text)
        messages.append(userMsg)

        isStreaming = true
        let assistantMsg = Message(role: .assistant, content: "")
        messages.append(assistantMsg)

        Task {
            await APIClient.shared.sendMessage(text, botId: nil) { event in
                Task { @MainActor in
                    self.applyStreamEvent(event)
                }
            }
            isStreaming = false
        }
    }

    @MainActor
    private func applyStreamEvent(_ event: StreamEvent) {
        switch event {
        case .contentDelta(let text):
            if !messages.isEmpty, messages.last?.role == .assistant, messages.last?.type == .text {
                messages[messages.count - 1].content += text
            }
        case .thinking(let text):
            let thinkingMsg = Message(role: .assistant, content: text, type: .thinking)
            messages.insert(thinkingMsg, at: max(0, messages.count - 1))
        case .toolUse(_, let name, let input):
            let toolMsg = Message(role: .assistant, content: input, type: .toolUse, toolName: name)
            messages.insert(toolMsg, at: max(0, messages.count - 1))
        case .toolResult(_, let content):
            let resultMsg = Message(role: .assistant, content: content, type: .toolResult)
            messages.insert(resultMsg, at: max(0, messages.count - 1))
        case .messageStop:
            isStreaming = false
        case .error(let msg):
            if !messages.isEmpty, messages.last?.role == .assistant {
                messages[messages.count - 1].content = "Error: \(msg)"
            }
            isStreaming = false
        default:
            break
        }
    }
}

struct MessageBubble: View {
    let message: Message

    var body: some View {
        HStack {
            if message.role == .user { Spacer(minLength: 60) }

            VStack(alignment: .leading, spacing: 4) {
                switch message.type {
                case .thinking:
                    Label("Thinking", systemImage: "brain")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    Text(message.content)
                        .font(.callout)
                        .foregroundStyle(.secondary)
                        .italic()
                case .toolUse:
                    Label(message.toolName ?? "Tool", systemImage: "wrench")
                        .font(.caption.bold())
                        .foregroundStyle(.orange)
                    Text(message.content)
                        .font(.caption)
                        .fontDesign(.monospaced)
                case .toolResult:
                    Label("Result", systemImage: "checkmark.circle")
                        .font(.caption)
                        .foregroundStyle(.green)
                    Text(message.content)
                        .font(.caption)
                        .fontDesign(.monospaced)
                        .lineLimit(5)
                case .text:
                    Text(message.content)
                }
            }
            .padding(12)
            .background(backgroundColor)
            .clipShape(RoundedRectangle(cornerRadius: 16))

            if message.role == .assistant { Spacer(minLength: 60) }
        }
    }

    private var backgroundColor: Color {
        switch message.type {
        case .thinking: return Color.gray.opacity(0.2)
        case .toolUse: return Color.orange.opacity(0.1)
        case .toolResult: return Color.green.opacity(0.1)
        case .text:
            return message.role == .user ? .blue : Color.gray.opacity(0.15)
        }
    }
}
