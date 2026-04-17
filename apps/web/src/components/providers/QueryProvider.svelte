<!--
  Ubicación: `apps/web/src/components/providers/QueryProvider.svelte`

  Descripción: Provider de TanStack Query para toda la aplicación.
               Envuelve los children con QueryClientProvider.
               Configurado con SSR y staleTime por defecto.

  ADRs relacionados: 0022 (Frontend), 0021 (OpenAPI)
-->

<script lang="ts">
	import { QueryClient, QueryClientProvider } from "@tanstack/svelte-query";
	import type { Snippet } from "svelte";

	interface Props {
		children: Snippet;
	}

	let { children }: Props = $props();

	const queryClient = new QueryClient({
		defaultOptions: {
			queries: {
				staleTime: 1000 * 60 * 5, // 5 minutos
				refetchOnWindowFocus: false
			}
		}
	});
</script>

<QueryClientProvider client={queryClient}>
	{@render children()}
</QueryClientProvider>
