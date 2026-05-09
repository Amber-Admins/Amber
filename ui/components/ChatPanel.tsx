import { useEffect, useMemo, useState } from "react";
import type { ContextAssemblerScope } from "../constants/contextBudget";
import { chatAppendMessage, getChatHistory, type ChatMessage } from "../services/chat";
import { chatWithScope } from "../services/nodes";
import {
  getLlmModel,
  getLlmProvider,
  getLmStudioEndpoint,
  getOllamaEndpoint,
} from "../utils/settings";

type ChatPanelProps = {
  selectedNodeIds: string[];
  scope: ContextAssemblerScope;
};

function ChatPanel({ selectedNodeIds, scope }: ChatPanelProps) {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState("");
  const [isSending, setIsSending] = useState(false);
  const [status, setStatus] = useState("");

  useEffect(() => {
    let active = true;
    void (async () => {
      try {
        const history = await getChatHistory();
        if (!active) {
          return;
        }
        setMessages(history);
        setStatus("");
      } catch (error) {
        if (!active) {
          return;
        }
        setStatus(String(error));
      }
    })();
    return () => {
      active = false;
    };
  }, []);

  const canSend = useMemo(() => input.trim().length > 0 && !isSending, [input, isSending]);

  async function handleSend() {
    if (!canSend) {
      return;
    }

    const prompt = input.trim();
    setInput("");
    setStatus("");
    setIsSending(true);

    const userMsgId = crypto.randomUUID();
    const userMsg: ChatMessage = {
      id: userMsgId,
      role: "user",
      content: prompt,
      created_at: new Date().toISOString(),
    };

    setMessages((prev) => [...prev, userMsg]);

    try {
      await chatAppendMessage(userMsgId, "user", prompt);

      const provider = getLlmProvider();
      const endpoint = provider === "lmstudio" ? getLmStudioEndpoint() : getOllamaEndpoint();
      const model = getLlmModel();

      const aiResponse = await chatWithScope(
        selectedNodeIds,
        scope,
        provider,
        endpoint,
        model,
        prompt
      );

      const aiMsgId = crypto.randomUUID();
      const aiMsg: ChatMessage = {
        id: aiMsgId,
        role: "assistant",
        content: aiResponse,
        created_at: new Date().toISOString(),
      };

      setMessages((prev) => [...prev, aiMsg]);
      await chatAppendMessage(aiMsgId, "assistant", aiResponse);
    } catch (error) {
      setStatus(String(error));
    } finally {
      setIsSending(false);
    }
  }

  return (
    <section className="chat-panel">
      <div className="chat-thread">
        {messages.map((message) => (
          <article key={message.id} className={`chat-message chat-message-${message.role}`}>
            <header>{message.role}</header>
            <p>{message.content}</p>
          </article>
        ))}
      </div>
      <div className="chat-composer">
        <textarea
          value={input}
          onChange={(event) => setInput(event.target.value)}
          placeholder="Ask MindVault..."
          disabled={isSending}
        />
        <button type="button" onClick={() => void handleSend()} disabled={!canSend}>
          {isSending ? "Sending..." : "Send"}
        </button>
      </div>
      {status && <p className="chat-status">{status}</p>}
    </section>
  );
}

export default ChatPanel;
