<!--
  Ubicación: `apps/web/src/components/dashboard/StatsOverview.svelte`

  Descripción: Componente que obtiene estadísticas reales del backend
               usando TanStack Query. Muestra KPIs de usuarios, sesiones,
               health de API y estado de base de datos.

  ADRs relacionados: 0022 (Frontend), 0021 (OpenAPI), 0023 (i18n)
-->

<script lang="ts">
	import { QueryClient, QueryClientProvider } from "@tanstack/svelte-query";
	import StatsContent from "./StatsContent.svelte";

	const queryClient = new QueryClient({
		defaultOptions: {
			queries: {
				staleTime: 1000 * 60 * 5,
				refetchOnWindowFocus: false
			}
		}
	});
</script>

<QueryClientProvider client={queryClient}>
	<StatsContent />
</QueryClientProvider>
