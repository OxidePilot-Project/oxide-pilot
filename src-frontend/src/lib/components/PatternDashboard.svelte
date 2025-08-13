<script lang="ts">
  import { onMount } from "svelte";
  import { writable } from "svelte/store";

  interface SystemMetric {
    id: string;
    name: string;
    value: number;
    unit: string;
    status: "good" | "warning" | "critical";
    icon: string;
    change?: number;
  }

  interface SystemStatus {
    cpu: number;
    memory: number;
    disk: number;
    network: number;
    temperature: number;
    uptime: string;
  }

  const metrics = writable<SystemMetric[]>([]);
  const systemStatus = writable<SystemStatus | null>(null);
  let isLoading = true;
  let error: string | null = null;
  let activeTab = "overview";

  // Pattern Craft inspired background patterns
  let backgroundPattern = "linear-gradient(135deg, #667eea 0%, #764ba2 100%)";

  onMount(async () => {
    await loadSystemMetrics();
  });

  async function loadSystemMetrics() {
    try {
      isLoading = true;
      error = null;
      
      // Simulate API call to get system metrics
      const status: SystemStatus = {
        cpu: Math.floor(Math.random() * 100),
        memory: Math.floor(Math.random() * 100),
        disk: Math.floor(Math.random() * 100),
        network: Math.floor(Math.random() * 100),
        temperature: Math.floor(Math.random() * 100),
        uptime: "2 days, 4 hours"
      };
      
      systemStatus.set(status);
      
      // Convert to display metrics with Pattern Craft inspired styling
      const metricList: SystemMetric[] = [
        { 
          id: "cpu", 
          name: "CPU Usage", 
          value: status.cpu, 
          unit: "%", 
          status: getStatus(status.cpu, 80, 90),
          icon: "⚙️",
          change: Math.floor(Math.random() * 20) - 10
        },
        { 
          id: "memory", 
          name: "Memory", 
          value: status.memory, 
          unit: "%", 
          status: getStatus(status.memory, 80, 90),
          icon: "💾",
          change: Math.floor(Math.random() * 20) - 10
        },
        { 
          id: "disk", 
          name: "Disk Usage", 
          value: status.disk, 
          unit: "%", 
          status: getStatus(status.disk, 85, 95),
          icon: "💿",
          change: Math.floor(Math.random() * 20) - 10
        },
        { 
          id: "network", 
          name: "Network", 
          value: status.network, 
          unit: "Mbps", 
          status: getStatus(status.network, 70, 90),
          icon: "🌐",
          change: Math.floor(Math.random() * 20) - 10
        },
        { 
          id: "temperature", 
          name: "Temperature", 
          value: status.temperature, 
          unit: "°C", 
          status: getStatus(status.temperature, 70, 85),
          icon: "🌡️",
          change: Math.floor(Math.random() * 20) - 10
        }
      ];
      
      metrics.set(metricList);
    } catch (err) {
      error = "Failed to load system metrics";
      console.error(err);
    } finally {
      isLoading = false;
    }
  }

  function getStatus(value: number, warningThreshold: number, criticalThreshold: number): "good" | "warning" | "critical" {
    if (value >= criticalThreshold) return "critical";
    if (value >= warningThreshold) return "warning";
    return "good";
  }

  function refreshData() {
    loadSystemMetrics();
  }

  function switchTab(tab: string) {
    activeTab = tab;
  }

  // Function to generate Pattern Craft inspired background
  function generateBackgroundPattern() {
    const patterns = [
      "linear-gradient(135deg, #667eea 0%, #764ba2 100%)",
      "linear-gradient(135deg, #f093fb 0%, #f5576c 100%)",
      "linear-gradient(135deg, #4facfe 0%, #00f2fe 100%)",
      "linear-gradient(135deg, #43e97b 0%, #38f9d7 100%)",
      "linear-gradient(135deg, #fa709a 0%, #fee140 100%)"
    ];
    return patterns[Math.floor(Math.random() * patterns.length)];
  }

  // Initialize with a random pattern
  backgroundPattern = generateBackgroundPattern();
</script>

<div class="pattern-dashboard" style="background: {backgroundPattern};">
  <div class="dashboard-container">
    <div class="dashboard-header">
      <div class="header-content">
        <h1>🛡️ Oxide Pilot Dashboard</h1>
        <p>AI-Powered System Monitoring & Management</p>
      </div>
      <div class="header-actions">
        <button class="action-button refresh" on:click={refreshData} disabled={isLoading} title="Refresh data">
          {#if isLoading}
            🔄 Refreshing...
          {:else}
            🔄 Refresh
          {/if}
        </button>
        <button class="action-button change-pattern" on:click={() => backgroundPattern = generateBackgroundPattern()} title="Change background pattern">
          🎨
        </button>
      </div>
    </div>

    <div class="dashboard-tabs">
      <button class:active={activeTab === "overview"} on:click={() => switchTab("overview")}>Overview</button>
      <button class:active={activeTab === "performance"} on:click={() => switchTab("performance")}>Performance</button>
      <button class:active={activeTab === "security"} on:click={() => switchTab("security")}>Security</button>
      <button class:active={activeTab === "ai-insights"} on:click={() => switchTab("ai-insights")}>AI Insights</button>
    </div>

    {#if error}
      <div class="error-message">
        ❌ {error}
      </div>
    {/if}

    <div class="dashboard-content">
      {#if activeTab === "overview"}
        <div class="overview-section">
          {#if isLoading && !$systemStatus}
            <div class="loading-skeleton">
              <div class="skeleton-card"></div>
              <div class="skeleton-card"></div>
              <div class="skeleton-card"></div>
              <div class="skeleton-card"></div>
              <div class="skeleton-card"></div>
            </div>
          {:else if $systemStatus}
            <div class="metrics-grid">
              {#each $metrics as metric (metric.id)}
                <div class="metric-card {metric.status}">
                  <div class="metric-header">
                    <span class="metric-icon">{metric.icon}</span>
                    <span class="metric-name">{metric.name}</span>
                  </div>
                  <div class="metric-value">
                    <span class="value">{metric.value}</span>
                    <span class="unit">{metric.unit}</span>
                  </div>
                  <div class="metric-progress">
                    <div class="progress-bar" style="width: {metric.value}%"></div>
                  </div>
                  {#if metric.change !== undefined}
                    <div class="metric-change {metric.change >= 0 ? 'positive' : 'negative'}">
                      {metric.change >= 0 ? '↑' : '↓'} {Math.abs(metric.change)}%
                    </div>
                  {/if}
                </div>
              {/each}
            </div>

            <div class="system-info-grid">
              <div class="info-card">
                <div class="info-icon">⏱️</div>
                <div class="info-content">
                  <h3>Uptime</h3>
                  <p>{$systemStatus.uptime}</p>
                </div>
              </div>
              <div class="info-card success">
                <div class="info-icon">🛡️</div>
                <div class="info-content">
                  <h3>Security Status</h3>
                  <p>All systems secure</p>
                </div>
              </div>
              <div class="info-card success">
                <div class="info-icon">📈</div>
                <div class="info-content">
                  <h3>Performance</h3>
                  <p>Optimal</p>
                </div>
              </div>
              <div class="info-card">
                <div class="info-icon">🧠</div>
                <div class="info-content">
                  <h3>AI Status</h3>
                  <p>Active & Learning</p>
                </div>
              </div>
            </div>
          {/if}
        </div>
      {:else if activeTab === "performance"}
        <div class="performance-section">
          <h2>📊 Performance Analytics</h2>
          <div class="chart-placeholder">
            <div class="chart-container">
              <div class="chart-grid"></div>
              <div class="chart-line"></div>
              <div class="chart-data-points">
                {#each [30, 45, 60, 40, 70, 55, 80] as value, i}
                  <div class="data-point" style="left: {i * 14}%; bottom: {value}%;"></div>
                {/each}
              </div>
            </div>
            <p>CPU Usage Over Time</p>
          </div>
        </div>
      {:else if activeTab === "security"}
        <div class="security-section">
          <h2>🔒 Security Overview</h2>
          <div class="security-grid">
            <div class="security-card">
              <div class="security-icon">🛡️</div>
              <h3>Threat Detection</h3>
              <p class="status success">Active</p>
              <p>No threats detected in the last 24 hours</p>
            </div>
            <div class="security-card">
              <div class="security-icon">🔔</div>
              <h3>Real-time Monitoring</h3>
              <p class="status success">Enabled</p>
              <p>Monitoring 127 processes</p>
            </div>
            <div class="security-card">
              <div class="security-icon">🔍</div>
              <h3>Behavioral Analysis</h3>
              <p class="status warning">Learning</p>
              <p>Building baseline behavior profile</p>
            </div>
            <div class="security-card">
              <div class="security-icon">🔄</div>
              <h3>Automatic Updates</h3>
              <p class="status success">Enabled</p>
              <p>Last update: Today at 02:30 AM</p>
            </div>
          </div>
        </div>
      {:else if activeTab === "ai-insights"}
        <div class="ai-insights-section">
          <h2>🤖 AI-Powered Insights</h2>
          <div class="insights-container">
            <div class="insight-card">
              <div class="insight-icon">💡</div>
              <div class="insight-content">
                <h3>Performance Optimization</h3>
                <p>Your system is running efficiently. Consider disabling unnecessary startup programs to improve boot time by up to 15%.</p>
              </div>
            </div>
            <div class="insight-card">
              <div class="insight-icon">🔋</div>
              <div class="insight-content">
                <h3>Battery Life</h3>
                <p>Battery health is at 92%. Your usage patterns suggest 6-8 hours of battery life with current settings.</p>
              </div>
            </div>
            <div class="insight-card">
              <div class="insight-icon">🧹</div>
              <div class="insight-content">
                <h3>Maintenance Recommendation</h3>
                <p>Clear temporary files and browser cache to free up 2.3GB of storage space.</p>
              </div>
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .pattern-dashboard {
    min-height: 100%;
    padding: 20px;
    background-size: 400% 400%;
    animation: gradientShift 15s ease infinite;
  }

  @keyframes gradientShift {
    0% { background-position: 0% 50%; }
    50% { background-position: 100% 50%; }
    100% { background-position: 0% 50%; }
  }

  .dashboard-container {
    background: rgba(255, 255, 255, 0.95);
    border-radius: 20px;
    box-shadow: 0 20px 50px rgba(0, 0, 0, 0.15);
    overflow: hidden;
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.2);
  }

  .dashboard-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 30px;
    background: rgba(26, 115, 232, 0.05);
    border-bottom: 1px solid rgba(0, 0, 0, 0.05);
  }

  .header-content h1 {
    margin: 0 0 8px 0;
    color: #2c3e50;
    font-size: 28px;
    font-weight: 700;
  }

  .header-content p {
    margin: 0;
    color: #7f8c8d;
    font-size: 16px;
  }

  .header-actions {
    display: flex;
    gap: 12px;
  }

  .action-button {
    background: rgba(255, 255, 255, 0.8);
    border: 1px solid rgba(0, 0, 0, 0.1);
    color: #2c3e50;
    padding: 12px 20px;
    border-radius: 12px;
    cursor: pointer;
    font-weight: 500;
    transition: all 0.3s ease;
    display: flex;
    align-items: center;
    gap: 8px;
    backdrop-filter: blur(5px);
  }

  .action-button:hover:not(:disabled) {
    background: rgba(255, 255, 255, 1);
    transform: translateY(-2px);
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.1);
  }

  .action-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
    transform: none;
  }

  .refresh {
    background: #1a73e8;
    color: white;
    border-color: #1a73e8;
  }

  .refresh:hover:not(:disabled) {
    background: #1557b0;
  }

  .change-pattern {
    padding: 12px;
    border-radius: 50%;
    width: 48px;
    height: 48px;
  }

  .dashboard-tabs {
    display: flex;
    border-bottom: 1px solid rgba(0, 0, 0, 0.05);
    background: rgba(255, 255, 255, 0.7);
  }

  .dashboard-tabs button {
    flex: 1;
    padding: 18px 20px;
    border: none;
    background: transparent;
    color: #7f8c8d;
    font-size: 16px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.3s ease;
    position: relative;
  }

  .dashboard-tabs button:hover {
    color: #2c3e50;
    background: rgba(0, 0, 0, 0.02);
  }

  .dashboard-tabs button.active {
    color: #1a73e8;
  }

  .dashboard-tabs button.active::after {
    content: '';
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    height: 3px;
    background: #1a73e8;
    border-radius: 3px 3px 0 0;
  }

  .dashboard-content {
    padding: 30px;
  }

  .error-message {
    background: #fce8e6;
    color: #c5221f;
    padding: 15px 20px;
    border-radius: 12px;
    margin-bottom: 25px;
    border: 1px solid #f9dedc;
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .loading-skeleton {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 25px;
    margin-bottom: 40px;
  }

  .skeleton-card {
    background: rgba(255, 255, 255, 0.7);
    border-radius: 16px;
    height: 160px;
    animation: pulse 1.5s ease-in-out infinite alternate;
    border: 1px solid rgba(0, 0, 0, 0.05);
  }

  @keyframes pulse {
    0% {
      background: rgba(255, 255, 255, 0.7);
    }
    100% {
      background: rgba(255, 255, 255, 0.9);
    }
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 25px;
    margin-bottom: 40px;
  }

  .metric-card {
    background: rgba(255, 255, 255, 0.9);
    border-radius: 16px;
    padding: 25px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.05);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.3);
    transition: all 0.3s ease;
    position: relative;
    overflow: hidden;
  }

  .metric-card:hover {
    transform: translateY(-5px);
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.1);
  }

  .metric-card::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 4px;
  }

  .metric-card.good::before {
    background: linear-gradient(90deg, #4caf50, #8bc34a);
  }

  .metric-card.warning::before {
    background: linear-gradient(90deg, #ff9800, #ffc107);
  }

  .metric-card.critical::before {
    background: linear-gradient(90deg, #f44336, #e91e63);
  }

  .metric-header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 20px;
  }

  .metric-icon {
    font-size: 24px;
  }

  .metric-name {
    font-weight: 600;
    color: #2c3e50;
    font-size: 18px;
  }

  .metric-value {
    display: flex;
    align-items: baseline;
    margin-bottom: 20px;
  }

  .value {
    font-size: 36px;
    font-weight: 700;
    color: #2c3e50;
    line-height: 1;
  }

  .unit {
    font-size: 18px;
    color: #7f8c8d;
    margin-left: 8px;
  }

  .metric-progress {
    height: 10px;
    background: #ecf0f1;
    border-radius: 5px;
    overflow: hidden;
    margin-bottom: 15px;
  }

  .progress-bar {
    height: 100%;
    border-radius: 5px;
    transition: width 0.5s ease;
  }

  .metric-card.good .progress-bar {
    background: linear-gradient(90deg, #4caf50, #8bc34a);
  }

  .metric-card.warning .progress-bar {
    background: linear-gradient(90deg, #ff9800, #ffc107);
  }

  .metric-card.critical .progress-bar {
    background: linear-gradient(90deg, #f44336, #e91e63);
  }

  .metric-change {
    font-size: 14px;
    font-weight: 600;
  }

  .metric-change.positive {
    color: #4caf50;
  }

  .metric-change.negative {
    color: #f44336;
  }

  .system-info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 25px;
  }

  .info-card {
    background: rgba(255, 255, 255, 0.9);
    border-radius: 16px;
    padding: 25px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.05);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.3);
    display: flex;
    gap: 20px;
    transition: all 0.3s ease;
  }

  .info-card:hover {
    transform: translateY(-3px);
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.1);
  }

  .info-card.success {
    border-left: 4px solid #4caf50;
  }

  .info-icon {
    font-size: 32px;
    flex-shrink: 0;
  }

  .info-content h3 {
    margin: 0 0 10px 0;
    color: #2c3e50;
    font-size: 18px;
  }

  .info-content p {
    margin: 0;
    color: #7f8c8d;
    font-size: 16px;
  }

  .status {
    font-weight: 600;
  }

  .status.success {
    color: #4caf50;
  }

  .status.warning {
    color: #ff9800;
  }

  .performance-section h2,
  .security-section h2,
  .ai-insights-section h2 {
    color: #2c3e50;
    margin: 0 0 30px 0;
    font-size: 28px;
    font-weight: 700;
  }

  .chart-placeholder {
    background: rgba(255, 255, 255, 0.9);
    border-radius: 16px;
    padding: 30px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.05);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.3);
  }

  .chart-container {
    position: relative;
    height: 300px;
    background: #f8f9fa;
    border-radius: 12px;
    margin-bottom: 20px;
    overflow: hidden;
  }

  .chart-grid {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-image: 
      linear-gradient(rgba(0, 0, 0, 0.05) 1px, transparent 1px),
      linear-gradient(90deg, rgba(0, 0, 0, 0.05) 1px, transparent 1px);
    background-size: 20px 20px;
  }

  .chart-line {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: #1a73e8;
  }

  .chart-data-points {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
  }

  .data-point {
    position: absolute;
    width: 12px;
    height: 12px;
    background: #1a73e8;
    border-radius: 50%;
    transform: translate(-50%, 50%);
    box-shadow: 0 0 0 4px rgba(26, 115, 232, 0.2);
  }

  .chart-placeholder p {
    text-align: center;
    color: #7f8c8d;
    font-size: 16px;
    margin: 0;
  }

  .security-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 25px;
  }

  .security-card {
    background: rgba(255, 255, 255, 0.9);
    border-radius: 16px;
    padding: 30px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.05);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.3);
    text-align: center;
    transition: all 0.3s ease;
  }

  .security-card:hover {
    transform: translateY(-5px);
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.1);
  }

  .security-icon {
    font-size: 40px;
    margin-bottom: 20px;
  }

  .security-card h3 {
    color: #2c3e50;
    margin: 0 0 15px 0;
    font-size: 20px;
  }

  .security-card p {
    color: #7f8c8d;
    margin: 0 0 10px 0;
    font-size: 16px;
    line-height: 1.5;
  }

  .insights-container {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 25px;
  }

  .insight-card {
    background: rgba(255, 255, 255, 0.9);
    border-radius: 16px;
    padding: 30px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.05);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.3);
    display: flex;
    gap: 20px;
    transition: all 0.3s ease;
  }

  .insight-card:hover {
    transform: translateY(-5px);
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.1);
  }

  .insight-icon {
    font-size: 32px;
    flex-shrink: 0;
  }

  .insight-content h3 {
    color: #2c3e50;
    margin: 0 0 15px 0;
    font-size: 20px;
  }

  .insight-content p {
    color: #7f8c8d;
    margin: 0;
    font-size: 16px;
    line-height: 1.6;
  }

  @media (max-width: 768px) {
    .pattern-dashboard {
      padding: 10px;
    }

    .dashboard-header {
      flex-direction: column;
      gap: 20px;
      align-items: stretch;
      padding: 20px;
    }

    .header-actions {
      justify-content: center;
    }

    .dashboard-content {
      padding: 20px;
    }

    .metrics-grid,
    .system-info-grid,
    .security-grid,
    .insights-container {
      grid-template-columns: 1fr;
    }

    .info-card,
    .security-card,
    .insight-card {
      flex-direction: column;
      text-align: center;
    }

    .dashboard-tabs {
      flex-wrap: wrap;
    }

    .dashboard-tabs button {
      flex: 1 1 50%;
      text-align: center;
    }
  }
</style>