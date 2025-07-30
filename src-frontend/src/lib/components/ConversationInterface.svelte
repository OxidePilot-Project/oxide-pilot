<script lang="ts">
import { invoke } from "@tauri-apps/api/tauri";
import { writable } from "svelte/store";

interface Message {
  id: string;
  sender: "user" | "agent" | "status";
  text: string;
  timestamp: Date;
}

const messages = writable<Message[]>([]);
let newMessage = "";
let isProcessing = false;

async function _sendMessage() {
  if (newMessage.trim() && !isProcessing) {
    isProcessing = true;
    const userText = newMessage;
    newMessage = "";

    messages.update((msgs) => [
      ...msgs,
      {
        id: Date.now().toString(),
        sender: "user",
        text: userText,
        timestamp: new Date(),
      },
    ]);

    messages.update((msgs) => [
      ...msgs,
      {
        id: `status-${Date.now().toString()}`,
        sender: "status",
        text: "Agent is thinking...",
        timestamp: new Date(),
      },
    ]);

    try {
      // Call the Tauri command to handle user input
      // This command will eventually call CopilotAgent::handle_user_input
      const agentResponse = await invoke("handle_user_input_command", {
        userInput: userText,
        context: {},
      });

      messages.update((msgs) => {
        const newMsgs = msgs.filter((msg) => msg.sender !== "status"); // Remove thinking status
        return [
          ...newMsgs,
          {
            id: Date.now().toString(),
            sender: "agent",
            text: agentResponse as string,
            timestamp: new Date(),
          },
        ];
      });
    } catch (error) {
      console.error("Error handling user input:", error);
      messages.update((msgs) => {
        const newMsgs = msgs.filter((msg) => msg.sender !== "status"); // Remove thinking status
        return [
          ...newMsgs,
          {
            id: Date.now().toString(),
            sender: "status",
            text: `Error: ${error}`,
            timestamp: new Date(),
          },
        ];
      });
    } finally {
      isProcessing = false;
    }
  }
}
</script>

<div class="conversation-container">
  <div class="messages">
    {#each $messages as message (message.id)}
      <div class="message {message.sender}">
        <p>{message.text}</p>
        <span>{message.timestamp.toLocaleTimeString()}</span>
      </div>
    {/each}
  </div>
  <div class="input-area">
    <input
      type="text"
      placeholder="Type your message..."
      bind:value={newMessage}
      on:keydown={e => { if (e.key === 'Enter') sendMessage(); }}
      disabled={isProcessing}
    />
    <button on:click={sendMessage} disabled={isProcessing}>Send</button>
  </div>
</div>

<style>
  .conversation-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    border: 1px solid #ccc;
    border-radius: 8px;
    overflow: hidden;
  }
  .messages {
    flex-grow: 1;
    padding: 10px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }
  .message {
    margin-bottom: 10px;
    padding: 8px 12px;
    border-radius: 15px;
    max-width: 70%;
  }
  .message.user {
    align-self: flex-end;
    background-color: #007bff;
    color: white;
    border-bottom-right-radius: 2px;
  }
  .message.agent {
    align-self: flex-start;
    background-color: #e2e2e2;
    color: #333;
    border-bottom-left-radius: 2px;
  }
  .message.status {
    align-self: center;
    background-color: #fff3cd;
    color: #856404;
    font-style: italic;
    font-size: 0.9em;
  }
  .message span {
    font-size: 0.7em;
    color: #666;
    display: block;
    margin-top: 5px;
  }
  .input-area {
    display: flex;
    padding: 10px;
    border-top: 1px solid #eee;
  }
  .input-area input {
    flex-grow: 1;
    padding: 8px;
    border: 1px solid #ddd;
    border-radius: 4px;
    margin-right: 10px;
  }
  .input-area button {
    padding: 8px 15px;
    background-color: #28a745;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }
  .input-area button:disabled {
    background-color: #cccccc;
    cursor: not-allowed;
  }
</style>