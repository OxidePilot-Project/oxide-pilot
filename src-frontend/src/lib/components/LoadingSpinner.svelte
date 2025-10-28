<script lang="ts">
  export let size: 'small' | 'medium' | 'large' = 'medium';
  export let color: string = '#3b82f6';
  export let label: string = 'Loading...';

  const sizeMap = {
    small: '20px',
    medium: '40px',
    large: '60px'
  };
</script>

<div class="loading-container" role="status" aria-live="polite" aria-label={label}>
  <div
    class="spinner"
    style="width: {sizeMap[size]}; height: {sizeMap[size]}; border-color: {color} transparent transparent transparent;"
  ></div>
  <span class="sr-only">{label}</span>
</div>

<style>
  .loading-container {
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 1rem;
  }

  .spinner {
    border: 3px solid;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border-width: 0;
  }
</style>
