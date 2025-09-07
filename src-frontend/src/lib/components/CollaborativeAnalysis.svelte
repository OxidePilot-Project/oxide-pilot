<script lang="ts">
  import { onMount } from "svelte";
  import { isTauri } from "$lib/utils/env";
  import { tauriInvoke } from "$lib/utils/tauri";

  export let userInput: string = "";
  export let taskType: string = "system_analysis";

  let isAnalyzing = false;
  let analysisResult: any = null;
  let error: string = "";

  interface CollaborativeResult {
    success: boolean;
    primary_response: string;
    secondary_responses: Record<string, string>;
    consensus_score: number;
    confidence: number;
    execution_plan: any;
    timestamp: string;
  }

  async function runCollaborativeAnalysis() {
    if (!userInput.trim()) {
      error = "Please enter a task or question";
      return;
    }

    isAnalyzing = true;
    error = "";
    analysisResult = null;

    try {
      if (isTauri) {
        const result = await tauriInvoke<string>("run_collaborative_analysis", {
          userInput: userInput.trim(),
          taskType: taskType
        });

        analysisResult = JSON.parse(result) as CollaborativeResult;
      } else {
        // Browser fallback
        analysisResult = {
          success: true,
          primary_response: `Web preview: Collaborative analysis for "${userInput}" would be performed by Gemini (Coordinator) and Qwen (Analyst) working together.`,
          secondary_responses: {
            "gemini_executor": "Execution plan would be created by Gemini Executor",
            "qwen_validator": "Results would be validated by Qwen Validator"
          },
          consensus_score: 0.85,
          confidence: 0.9,
          execution_plan: {
            plan: "Web preview execution plan",
            steps: ["Step 1: Analysis", "Step 2: Validation", "Step 3: Execution"],
            timestamp: new Date().toISOString()
          },
          timestamp: new Date().toISOString()
        };
      }
    } catch (err) {
      error = `Analysis failed: ${err}`;
      console.error("Collaborative analysis error:", err);
    } finally {
      isAnalyzing = false;
    }
  }

  function getConfidenceColor(confidence: number): string {
    if (confidence >= 0.8) return "#22c55e"; // green
    if (confidence >= 0.6) return "#f59e0b"; // yellow
    return "#ef4444"; // red
  }

  function getConsensusColor(score: number): string {
    if (score >= 0.8) return "#22c55e"; // green
    if (score >= 0.6) return "#f59e0b"; // yellow
    return "#ef4444"; // red
  }
</script>

<div class="collaborative-analysis">
  <div class="analysis-header">
    <h2>🤝 Collaborative LLM Analysis</h2>
    <p>Gemini and Qwen working together to solve complex tasks</p>
  </div>

  <div class="input-section">
    <div class="input-group">
      <label for="userInput">Task or Question:</label>
      <textarea
        id="userInput"
        bind:value={userInput}
        placeholder="Describe the task you want the AI agents to collaborate on..."
        rows="3"
        disabled={isAnalyzing}
      ></textarea>
    </div>

    <div class="input-group">
      <label for="taskType">Task Type:</label>
      <select id="taskType" bind:value={taskType} disabled={isAnalyzing}>
        <option value="system_analysis">System Analysis</option>
        <option value="security_assessment">Security Assessment</option>
        <option value="performance_optimization">Performance Optimization</option>
        <option value="troubleshooting">Troubleshooting</option>
        <option value="user_query">General Query</option>
      </select>
    </div>

    <button
      class="analyze-button"
      on:click={runCollaborativeAnalysis}
      disabled={isAnalyzing || !userInput.trim()}
    >
      {#if isAnalyzing}
        <div class="spinner"></div>
        Analyzing...
      {:else}
        🚀 Run Collaborative Analysis
      {/if}
    </button>
  </div>

  {#if error}
    <div class="error-message">
      <div class="error-icon">❌</div>
      <span>{error}</span>
    </div>
  {/if}

  {#if analysisResult}
    <div class="analysis-results">
      <div class="results-header">
        <h3>📊 Analysis Results</h3>
        <div class="metrics">
          <div class="metric">
            <span class="metric-label">Confidence:</span>
            <span
              class="metric-value"
              style="color: {getConfidenceColor(analysisResult.confidence)}"
            >
              {(analysisResult.confidence * 100).toFixed(1)}%
            </span>
          </div>
          <div class="metric">
            <span class="metric-label">Consensus:</span>
            <span
              class="metric-value"
              style="color: {getConsensusColor(analysisResult.consensus_score)}"
            >
              {(analysisResult.consensus_score * 100).toFixed(1)}%
            </span>
          </div>
        </div>
      </div>

      <div class="results-content">
        <!-- Primary Response (Coordinator) -->
        <div class="response-section primary">
          <div class="response-header">
            <h4>🎯 Primary Response (Coordinator)</h4>
            <span class="provider-badge gemini">Gemini</span>
          </div>
          <div class="response-content">
            {analysisResult.primary_response}
          </div>
        </div>

        <!-- Secondary Responses -->
        {#if Object.keys(analysisResult.secondary_responses).length > 0}
          <div class="response-section secondary">
            <h4>🔍 Specialized Analysis</h4>
            <div class="secondary-responses">
              {#each Object.entries(analysisResult.secondary_responses) as [provider, response]}
                <div class="secondary-response">
                  <div class="response-header">
                    <span class="provider-badge {provider.includes('qwen') ? 'qwen' : 'gemini'}">
                      {provider.includes('qwen') ? 'Qwen' : 'Gemini'}
                    </span>
                    <span class="role-label">
                      {provider.includes('analyst') ? 'Analyst' :
                       provider.includes('executor') ? 'Executor' :
                       provider.includes('validator') ? 'Validator' :
                       provider.includes('innovator') ? 'Innovator' : 'Specialist'}
                    </span>
                  </div>
                  <div class="response-content">
                    {response}
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Execution Plan -->
        {#if analysisResult.execution_plan}
          <div class="response-section execution">
            <div class="response-header">
              <h4>📋 Execution Plan</h4>
            </div>
            <div class="response-content">
              {#if typeof analysisResult.execution_plan === 'string'}
                <pre>{analysisResult.execution_plan}</pre>
              {:else if analysisResult.execution_plan.steps}
                <div class="execution-steps">
                  {#each analysisResult.execution_plan.steps as step, index}
                    <div class="execution-step">
                      <span class="step-number">{index + 1}</span>
                      <span class="step-content">{step}</span>
                    </div>
                  {/each}
                </div>
              {:else}
                <pre>{JSON.stringify(analysisResult.execution_plan, null, 2)}</pre>
              {/if}
            </div>
          </div>
        {/if}
      </div>

      <div class="results-footer">
        <small>Analysis completed at {new Date(analysisResult.timestamp).toLocaleString()}</small>
      </div>
    </div>
  {/if}
</div>

<style>
  .collaborative-analysis {
    max-width: 1000px;
    margin: 0 auto;
    padding: 20px;
    background: var(--color-surface);
    border-radius: 12px;
    box-shadow: var(--shadow-md);
    border: 1px solid rgba(0,0,0,0.06);
  }

  .analysis-header {
    text-align: center;
    margin-bottom: 30px;
  }

  .analysis-header h2 {
    color: var(--color-text);
    margin: 0 0 10px 0;
    font-size: 24px;
  }

  .analysis-header p {
    color: var(--color-muted);
    margin: 0;
    font-size: 14px;
  }

  .input-section {
    background: var(--color-surface);
    border-radius: 12px;
    padding: 20px;
    margin-bottom: 20px;
    box-shadow: var(--shadow-md);
    border: 1px solid rgba(0,0,0,0.06);
  }

  .input-group {
    margin-bottom: 16px;
  }

  .input-group label {
    display: block;
    margin-bottom: 8px;
    font-weight: 500;
    color: var(--color-text);
    font-size: 14px;
  }

  .input-group textarea,
  .input-group select {
    width: 100%;
    padding: 12px;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    font-size: 14px;
    font-family: inherit;
    background: var(--color-surface);
    color: var(--color-text);
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
    box-sizing: border-box;
  }

  .input-group textarea:hover,
  .input-group select:hover { border-color: #cbd5e1; }
  .input-group textarea:focus-visible,
  .input-group select:focus-visible {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59,130,246,0.2);
  }

  .input-group textarea:disabled,
  .input-group select:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .analyze-button {
    width: 100%;
    padding: 14px 20px;
    background: var(--color-primary);
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 0.15s ease, transform 0.05s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
  }

  .analyze-button:hover:not(:disabled) {
    background: #2563eb;
    transform: translateY(-1px);
  }
  .analyze-button:focus-visible { outline: 2px solid #3b82f6; outline-offset: 2px; }
  .analyze-button:active { transform: translateY(1px); }

  .analyze-button:disabled {
    background: #9ca3af;
    cursor: not-allowed;
    transform: none;
  }

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top: 2px solid white;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  .error-message {
    background: #fef2f2;
    border: 1px solid #fecaca;
    color: #dc2626;
    padding: 12px 16px;
    border-radius: 8px;
    margin-bottom: 20px;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .error-icon {
    font-size: 16px;
  }

  .analysis-results {
    background: var(--color-surface);
    border-radius: 12px;
    padding: 20px;
    box-shadow: var(--shadow-md);
  }

  .results-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
    padding-bottom: 16px;
    border-bottom: 1px solid rgba(0,0,0,0.08);
  }

  .results-header h3 {
    margin: 0;
    color: var(--color-text);
    font-size: 20px;
  }

  .metrics {
    display: flex;
    gap: 20px;
  }

  .metric {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }

  .metric-label {
    font-size: 12px;
    color: var(--color-muted);
    font-weight: 500;
  }

  .metric-value {
    font-size: 16px;
    font-weight: 700;
  }

  .response-section {
    margin-bottom: 24px;
    padding: 16px;
    border-radius: 8px;
    border: 1px solid rgba(0,0,0,0.08);
  }

  .response-section.primary {
    background: linear-gradient(135deg, #eff6ff 0%, #dbeafe 100%);
    border-color: #3b82f6;
  }

  .response-section.secondary {
    background: #f8fafc;
  }

  .response-section.execution {
    background: linear-gradient(135deg, #f0fdf4 0%, #dcfce7 100%);
    border-color: #22c55e;
  }

  .response-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  .response-header h4 {
    margin: 0;
    color: var(--color-text);
    font-size: 16px;
    font-weight: 600;
  }

  .provider-badge {
    padding: 4px 8px;
    border-radius: 12px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
  }

  .provider-badge.gemini {
    background: #3b82f6;
    color: white;
  }

  .provider-badge.qwen {
    background: #8b5cf6;
    color: white;
  }

  .role-label {
    font-size: 12px;
    color: var(--color-muted);
    font-weight: 500;
  }

  .response-content {
    color: var(--color-text);
    line-height: 1.6;
    font-size: 14px;
  }

  .response-content pre {
    background: rgba(0,0,0,0.05);
    padding: 12px;
    border-radius: 6px;
    overflow-x: auto;
    font-size: 13px;
    margin: 0;
  }

  .secondary-responses {
    display: grid;
    gap: 16px;
  }

  .secondary-response {
    background: white;
    padding: 12px;
    border-radius: 6px;
    border: 1px solid rgba(0,0,0,0.06);
  }

  .execution-steps {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .execution-step {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 0;
  }

  .step-number {
    background: var(--color-primary);
    color: white;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
    font-weight: 600;
    flex-shrink: 0;
  }

  .step-content {
    color: var(--color-text);
    font-size: 14px;
  }

  .results-footer {
    margin-top: 20px;
    padding-top: 16px;
    border-top: 1px solid rgba(0,0,0,0.08);
    text-align: center;
  }

  .results-footer small {
    color: var(--color-muted);
    font-size: 12px;
  }

  @media (max-width: 768px) {
    .collaborative-analysis {
      padding: 16px;
    }

    .results-header {
      flex-direction: column;
      gap: 16px;
      align-items: flex-start;
    }

    .metrics {
      width: 100%;
      justify-content: space-around;
    }

    .secondary-responses {
      grid-template-columns: 1fr;
    }
  }
</style>
