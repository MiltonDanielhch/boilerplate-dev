<script lang="ts">
	import { authStore } from "$lib/stores/auth.svelte";

	interface Props {
		permission: string;
		children?: import("svelte").Snippet;
	}

	let { permission, children }: Props = $props();

	const allowed = $derived.by(() => {
		const user = authStore.user;
		return user?.permissions?.includes(permission) ?? false;
	});
</script>

{#if allowed}
	{@render children?.()}
{/if}