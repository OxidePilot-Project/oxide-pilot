<script lang="ts">
import { createEventDispatcher } from "svelte";
import { fadeIn, hoverLift, ripple } from "$lib/actions/anime";
export const variant: "primary" | "outline" | "ghost" = "primary";
export const size: "sm" | "md" | "lg" = "md";
export const disabled: boolean = false;
export const loading: boolean = false;
export const type: "button" | "submit" | "reset" = "button";
export let title: string | undefined;
const dispatch = createEventDispatcher<{ click: MouseEvent }>();
function onClick(e: MouseEvent) {
  if (disabled || loading) {
    e.preventDefault();
    return;
  }
  dispatch("click", e);
}
</script>

<button
  class={`btn btn-${size} ${variant==='primary'?'btn-primary':variant==='outline'?'btn-outline':'btn-ghost'}`}
  {type}
  {title}
  aria-busy={loading ? 'true' : undefined}
  disabled={disabled || loading}
  on:click={onClick}
  use:fadeIn={{ y: 6, duration: 260 }}
  use:hoverLift={{ y: 2 }}
  use:ripple
>
  <slot />{#if loading}&nbsp;…{/if}
</button>
