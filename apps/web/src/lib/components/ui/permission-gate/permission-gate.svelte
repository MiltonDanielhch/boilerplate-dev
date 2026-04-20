<script lang="ts">
	import { get } from "svelte/store";
	import { userStore } from "$lib/stores/auth.svelte";

	interface Props {
		permission: string;
		children?: import("svelte").Snippet;
	}

	let { permission, children }: Props = $props();

	const allowed = $derived.by(() => {
		const user = get(userStore);
		return user?.permissions?.includes(permission) ?? false;
	});
</script>

{#if allowed}
	{@render children?.()}
{/if}