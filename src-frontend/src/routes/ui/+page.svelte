<script lang="ts">
import Button from "$lib/ui/Button.svelte";

let isLoading = false;
function toggleLoading() {
  isLoading = !isLoading;
}
</script>

<div class="container py-4 no-x-overflow">
  <h1 style="font-size: var(--fs-2xl); margin-bottom: var(--space-4);">UI Showcase · Buttons</h1>

  <section class="card" style="margin-bottom: var(--space-5);">
    <div class="card-header">
      <h2 class="truncate" style="font-size: var(--fs-xl);">Variants</h2>
      <div class="muted">fadeIn · hoverLift · ripple</div>
    </div>
    <div class="card-body">
      <div class="flex gap-3 wrap">
        <Button title="Primary button" on:click={() => {}}>Primary</Button>
        <Button title="Outline button" variant="outline">Outline</Button>
        <Button title="Ghost button" variant="ghost">Ghost</Button>
      </div>
    </div>
  </section>

  <section class="card" style="margin-bottom: var(--space-5);">
    <div class="card-header">
      <h2 class="truncate" style="font-size: var(--fs-xl);">Sizes</h2>
    </div>
    <div class="card-body">
      <div class="flex items-center gap-3 wrap">
        <Button title="Small button" size="sm">Small</Button>
        <Button title="Medium button" size="md">Medium</Button>
        <Button title="Large button" size="lg">Large</Button>
      </div>
    </div>
  </section>

  <section class="card" style="margin-bottom: var(--space-5);">
    <div class="card-header">
      <h2 class="truncate" style="font-size: var(--fs-xl);">Loading & Disabled</h2>
      <div class="flex gap-2">
        <Button title="Toggle loading state" variant="outline" on:click={toggleLoading}>{isLoading ? 'Set idle' : 'Set loading'}</Button>
      </div>
    </div>
    <div class="card-body">
      <div class="flex items-center gap-3 wrap">
        <Button title="Action button" loading={isLoading}>Action</Button>
        <Button title="Disabled button" disabled>Disabled</Button>
        <Button title="Outline action button" variant="outline" loading={isLoading}>Outline Action</Button>
        <Button title="Ghost action button" variant="ghost" loading={isLoading}>Ghost Action</Button>
      </div>
    </div>
  </section>

  <section class="card">
    <div class="card-header">
      <h2 class="truncate" style="font-size: var(--fs-xl);">Ripple Color Examples</h2>
      <div class="muted">override ripple via CSS var</div>
    </div>
    <div class="card-body">
      <div class="flex items-center gap-3 wrap">
        <div style="--ripple-color: rgba(255,255,255,0.6)">
          <Button title="Light ripple button">Light Ripple</Button>
        </div>
        <div style="--ripple-color: rgba(79,70,229,0.35)">
          <Button title="Indigo ripple button" variant="outline">Indigo Ripple</Button>
        </div>
        <div style="--ripple-color: rgba(16,185,129,0.35)">
          <Button title="Green ripple button" variant="ghost">Green Ripple</Button>
        </div>
      </div>
    </div>
  </section>
</div>
