<script lang="ts">
	import { authStore } from "$lib/stores/auth.svelte";

	interface Props {
		permission: string;
		children?: import("svelte").Snippet;
		fallback?: import("svelte").Snippet;
	}

	let { permission, children, fallback }: Props = $props();

	const allowed = $derived.by(() => {
		const user = authStore.user;
		// Si no hay usuario cargado, no mostrar nada (evita flash de contenido)
		if (!user) return false;
		return user.permissions?.includes(permission) ?? false;
	});
</script>

{#if allowed}
	{@render children?.()}
{:else if fallback}
	{@render fallback?.()}
{/if}