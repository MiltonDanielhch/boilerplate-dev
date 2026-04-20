<script lang="ts">
	import { cn } from "$lib/utils.js";
	import type { HTMLTableAttributes } from "svelte/elements";

	let { class: className, children, ...rest }: HTMLTableAttributes = $props();
</script>

<div class="relative w-full overflow-auto">
	<table class={cn("w-full caption-bottom text-sm", className)} {...rest}>
		{@render children?.()}
	</table>
</div>
