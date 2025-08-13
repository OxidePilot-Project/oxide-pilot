<script lang="ts">
import { writable } from "svelte/store";
import { isTauri } from "$lib/utils/env";

// Lazy-load Tauri invoke to avoid SSR importing '@tauri-apps/api/tauri'
type InvokeFn = <T = any>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
let invokeFn: InvokeFn | null = null;
async function tauriInvoke<T = any>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri) throw new Error("Not running in Tauri context");
  if (!invokeFn) {
    const mod = await import("@tauri-apps/api/tauri");
    invokeFn = mod.invoke as InvokeFn;
  }
  return invokeFn<T>(cmd, args);
}

interface Message {
  id: string;
  text: string;
  sender: "user" | "assistant";
  timestamp: Date;
  status?: "sending" | "delivered" | "error";
}

const messages = writable<Message[]>([]);
let inputText = "";
let isProcessing = false;
let charCount = 0;

async function sendMessage() {
  if (!inputText.trim() || isProcessing) return;

  const userMessage: Message = {
    id: Date.now().toString(),
    text: inputText.trim(),
    sender: "user",
    timestamp: new Date(),
    status: "sending"
  };

  messages.update(msgs => [...msgs, userMessage]);
  const currentInput = inputText;
  inputText = "";
  charCount = 0;
  isProcessing = true;

  try {
    let response: string;
    if (isTauri) {
      response = await tauriInvoke<string>("handle_user_input_command", {
        user_input: currentInput
      });
    } else {
      // Browser/SSR fallback to avoid crashes during web preview/E2E
      response = `Web preview: I received your message: "${currentInput}"`;
    }

    // Update user message status
    messages.update(msgs => {
      const updated = [...msgs];
      const userMsgIndex = updated.findIndex(msg => msg.id === userMessage.id);
      if (userMsgIndex !== -1) {
        updated[userMsgIndex].status = "delivered";
      }
      return updated;
    });

    const assistantMessage: Message = {
      id: (Date.now() + 1).toString(),
      text: response as string,
      sender: "assistant",
      timestamp: new Date()
    };

    messages.update(msgs => [...msgs, assistantMessage]);
  } catch (error) {
    // Update user message status to error
    messages.update(msgs => {
      const updated = [...msgs];
      const userMsgIndex = updated.findIndex(msg => msg.id === userMessage.id);
      if (userMsgIndex !== -1) {
        updated[userMsgIndex].status = "error";
        updated[userMsgIndex].text += `\n\n⚠️ Error: ${error}`;
      }
      return updated;
    });
  } finally {
    isProcessing = false;
  }
}

function handleKeyPress(event: KeyboardEvent) {
  if (event.key === "Enter" && !event.shiftKey) {
    event.preventDefault();
    sendMessage();
  }
}

function handleInput(event: Event) {
  const target = event.target as HTMLTextAreaElement;
  inputText = target.value;
  charCount = inputText.length;
}
</script>

<div class="conversation-interface">
  <div class="messages-header">
    <h2>💬 Conversation with Oxide Pilot</h2>
    <div class="header-actions">
      <button class="action-button" on:click={() => messages.set([])}>
        🧹 Clear Chat
      </button>
    </div>
  </div>

  <div class="messages-container">
    {#if $messages.length === 0}
      <div class="welcome-message">
        <div class="welcome-icon">🤖</div>
        <h3>Welcome to Oxide Pilot</h3>
        <p>Ask me anything about your system, and I'll help you manage and optimize it.</p>
        <div class="suggestion-chips">
          <button class="chip" on:click={() => {inputText = "How can you help me optimize my system?"; charCount = inputText.length;}}>How can you help me optimize my system?</button>
          <button class="chip" on:click={() => {inputText = "Check my system security status"; charCount = inputText.length;}}>Check my system security status</button>
          <button class="chip" on:click={() => {inputText = "Show me my current CPU usage"; charCount = inputText.length;}}>Show me my current CPU usage</button>
        </div>
      </div>
    {/if}

    {#each $messages as message (message.id)}
      <div class="message {message.sender} {message.status || ''}">
        <div class="message-avatar">
          {#if message.sender === "user"}
            👤
          {:else}
            🤖
          {/if}
        </div>
        <div class="message-content">
          <div class="message-text">{message.text}</div>
          <div class="message-meta">
            <div class="message-time">
              {message.timestamp.toLocaleTimeString([], {hour: '2-digit', minute:'2-digit'})}
            </div>
            {#if message.status === "sending"}
              <div class="message-status sending">Sending...</div>
            {:else if message.status === "delivered"}
              <div class="message-status delivered">✓ Delivered</div>
            {:else if message.status === "error"}
              <div class="message-status error">✗ Error</div>
            {/if}
          </div>
        </div>
      </div>
    {/each}

    {#if isProcessing}
      <div class="message assistant processing">
        <div class="message-avatar">🤖</div>
        <div class="message-content">
          <div class="message-text">
            <div class="typing-indicator">
              <span></span>
              <span></span>
              <span></span>
            </div>
          </div>
        </div>
      </div>
    {/if}
  </div>

  <div class="input-container">
    <div class="input-area">
      <textarea
        bind:value={inputText}
        on:keypress={handleKeyPress}
        on:input={handleInput}
        placeholder="Message Oxide Pilot... (Press Enter to send, Shift+Enter for new line)"
        disabled={isProcessing}
        rows="3"
      ></textarea>
      <div class="input-footer">
        <div class="char-count {charCount > 1000 ? 'warning' : ''}">
          {charCount}/1000
        </div>
        <button
          on:click={sendMessage}
          disabled={!inputText.trim() || isProcessing || charCount > 1000}
          class="send-button"
          title="Send message"
        >
          {#if isProcessing}
            🔄
          {:else}
            📤
          {/if}
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  .conversation-interface {
    display: flex;
    flex-direction: column;
    height: 100%;
    max-height: calc(100vh - 200px);
    background: linear-gradient(135deg, #f5f7fa 0%, #e4e7f1 100%);
  }

  .messages-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px;
    background: rgba(255, 255, 255, 0.8);
    backdrop-filter: blur(10px);
    border-bottom: 1px solid #e1e8ed;
  }

  .messages-header h2 {
    margin: 0;
    color: #2c3e50;
    font-size: 20px;
    font-weight: 600;
  }

  .header-actions {
    display: flex;
    gap: 10px;
  }

  .action-button {
    background: #e8f0fe;
    border: 1px solid #d2e3fc;
    color: #1a73e8;
    padding: 8px 16px;
    border-radius: 20px;
    font-size: 14px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .action-button:hover {
    background: #d2e3fc;
    transform: translateY(-1px);
  }

  .messages-container {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 15px;
  }

  .welcome-message {
    text-align: center;
    padding: 40px 20px;
    background: rgba(255, 255, 255, 0.7);
    border-radius: 16px;
    margin: 20px auto;
    max-width: 600px;
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.3);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.05);
  }

  .welcome-icon {
    font-size: 48px;
    margin-bottom: 20px;
  }

  .welcome-message h3 {
    color: #2c3e50;
    margin: 0 0 10px 0;
    font-size: 24px;
  }

  .welcome-message p {
    color: #7f8c8d;
    margin: 0 0 30px 0;
    font-size: 16px;
    line-height: 1.6;
  }

  .suggestion-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    justify-content: center;
  }

  .chip {
    background: #e8f0fe;
    border: 1px solid #d2e3fc;
    color: #1a73e8;
    padding: 10px 16px;
    border-radius: 20px;
    font-size: 14px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .chip:hover {
    background: #d2e3fc;
    transform: translateY(-2px);
  }

  .message {
    display: flex;
    gap: 12px;
    max-width: 85%;
    animation: messageEnter 0.3s ease;
  }

  @keyframes messageEnter {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .message.user {
    align-self: flex-end;
  }

  .message.assistant {
    align-self: flex-start;
  }

  .message-avatar {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: #e8f0fe;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 18px;
    flex-shrink: 0;
  }

  .message.user .message-avatar {
    background: #1a73e8;
    color: white;
  }

  .message-content {
    background: #f1f3f4;
    border-radius: 18px;
    padding: 12px 16px;
    position: relative;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
  }

  .message.user .message-content {
    background: #1a73e8;
    color: white;
  }

  .message.assistant .message-content {
    background: #ffffff;
    border: 1px solid #e1e8ed;
  }

  .message.sending .message-content {
    opacity: 0.7;
  }

  .message.error .message-content {
    background: #fce8e6;
    border-color: #f9dedc;
    color: #c5221f;
  }

  .message-text {
    font-size: 14px;
    line-height: 1.5;
    word-wrap: break-word;
    white-space: pre-wrap;
  }

  .message.user .message-text {
    color: white;
  }

  .message.error .message-text {
    color: #c5221f;
  }

  .message-meta {
    display: flex;
    justify-content: space-between;
    margin-top: 8px;
  }

  .message-time {
    font-size: 11px;
    opacity: 0.7;
  }

  .message.user .message-time {
    color: rgba(255, 255, 255, 0.8);
  }

  .message-status {
    font-size: 11px;
    font-weight: 500;
  }

  .message-status.delivered {
    color: #4caf50;
  }

  .message-status.sending {
    color: #ff9800;
  }

  .message-status.error {
    color: #f44336;
  }

  .typing-indicator {
    display: flex;
    gap: 4px;
    align-items: center;
  }

  .typing-indicator span {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #1a73e8;
    animation: typing 1.4s infinite ease-in-out;
  }

  .typing-indicator span:nth-child(1) {
    animation-delay: -0.32s;
  }

  .typing-indicator span:nth-child(2) {
    animation-delay: -0.16s;
  }

  @keyframes typing {
    0%, 80%, 100% {
      transform: scale(0);
      opacity: 0.5;
    }
    40% {
      transform: scale(1);
      opacity: 1;
    }
  }

  .input-container {
    padding: 20px;
    border-top: 1px solid #e1e8ed;
    background: rgba(255, 255, 255, 0.9);
    backdrop-filter: blur(10px);
  }

  .input-area {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  textarea {
    flex: 1;
    border: 2px solid #dadce0;
    border-radius: 18px;
    padding: 16px 20px;
    font-size: 14px;
    font-family: inherit;
    resize: none;
    transition: border-color 0.2s ease;
    min-height: 80px;
    max-height: 200px;
  }

  textarea:focus {
    outline: none;
    border-color: #1a73e8;
    box-shadow: 0 0 0 3px rgba(26, 115, 232, 0.1);
  }

  textarea:disabled {
    background: #f8f9fa;
    color: #9aa0a6;
  }

  .input-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .char-count {
    font-size: 12px;
    color: #7f8c8d;
  }

  .char-count.warning {
    color: #f44336;
  }

  .send-button {
    background: #1a73e8;
    color: white;
    border: none;
    border-radius: 50%;
    width: 48px;
    height: 48px;
    cursor: pointer;
    font-size: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
    box-shadow: 0 4px 12px rgba(26, 115, 232, 0.3);
  }

  .send-button:hover:not(:disabled) {
    background: #1557b0;
    transform: scale(1.05);
  }

  .send-button:disabled {
    background: #f1f3f4;
    color: #9aa0a6;
    cursor: not-allowed;
    transform: none;
    box-shadow: none;
  }

  /* Scrollbar styling */
  .messages-container::-webkit-scrollbar {
    width: 8px;
  }

  .messages-container::-webkit-scrollbar-track {
    background: #f1f3f4;
    border-radius: 4px;
  }

  .messages-container::-webkit-scrollbar-thumb {
    background: #dadce0;
    border-radius: 4px;
  }

  .messages-container::-webkit-scrollbar-thumb:hover {
    background: #bdc1c6;
  }

  @media (max-width: 768px) {
    .message {
      max-width: 90%;
    }

    .messages-header {
      flex-direction: column;
      gap: 15px;
      align-items: stretch;
    }

    .header-actions {
      justify-content: center;
    }

    .welcome-message {
      padding: 20px 15px;
    }

    .suggestion-chips {
      flex-direction: column;
      align-items: center;
    }

    .chip {
      width: 100%;
    }
  }
</style>