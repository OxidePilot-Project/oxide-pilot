<script lang="ts">
  import Button from '$lib/ui/Button.svelte';
  let isLoading = false;
  function toggleLoading(){ isLoading = !isLoading; }
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
        <Button on:click={() => {}}>Primary</Button>
        <Button variant="outline">Outline</Button>
        <Button variant="ghost">Ghost</Button>
      </div>
    </div>
  </section>

  <section class="card" style="margin-bottom: var(--space-5);">
    <div class="card-header">
      <h2 class="truncate" style="font-size: var(--fs-xl);">Sizes</h2>
    </div>
    <div class="card-body">
      <div class="flex items-center gap-3 wrap">
        <Button size="sm">Small</Button>
        <Button size="md">Medium</Button>
        <Button size="lg">Large</Button>
      </div>
    </div>
  </section>

  <section class="card" style="margin-bottom: var(--space-5);">
    <div class="card-header">
      <h2 class="truncate" style="font-size: var(--fs-xl);">Loading & Disabled</h2>
      <div class="flex gap-2">
        <Button variant="outline" on:click={toggleLoading}>{isLoading ? 'Set idle' : 'Set loading'}</Button>
      </div>
    </div>
    <div class="card-body">
      <div class="flex items-center gap-3 wrap">
        <Button loading={isLoading}>Action</Button>
        <Button disabled>Disabled</Button>
        <Button variant="outline" loading={isLoading}>Outline Action</Button>
        <Button variant="ghost" loading={isLoading}>Ghost Action</Button>
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
        <Button style="--ripple-color: rgba(255,255,255,0.6)">Light Ripple</Button>
        <Button variant="outline" style="--ripple-color: rgba(79,70,229,0.35)">Indigo Ripple</Button>
        <Button variant="ghost" style="--ripple-color: rgba(16,185,129,0.35)">Green Ripple</Button>
      </div>
    </div>
  </section>
</div>
