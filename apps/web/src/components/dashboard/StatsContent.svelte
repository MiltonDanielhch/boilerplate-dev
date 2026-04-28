<!--
  Ubicación: `apps/web/src/components/dashboard/StatsContent.svelte`

  Descripción: Contenido de estadísticas que usa createQuery.
               Debe estar dentro de QueryClientProvider.

  ADRs relacionados: 0022 (Frontend), 0021 (OpenAPI), 0023 (i18n)
-->

<script lang="ts">
	import { onMount } from "svelte";
	import KpiCard from "./KpiCard.svelte";
	import { listUsers } from "$lib/api/users";
	import { isTauri } from "$lib/tauri";
	import * as m from "$lib/paraglide/messages.js";

	let usersTotal = $state<number | null>(null);
	let usersLoading = $state(true);
	let usersError = $state<Error | null>(null);

	let healthData = $state<unknown | null>(null);
	let healthLoading = $state(true);
	let healthError = $state<Error | null>(null);

	async function fetchUsers() {
		// En Tauri, no hay API HTTP
		if (isTauri()) {
			usersTotal = 1;
			usersLoading = false;
			return;
		}
		
		try {
			usersLoading = true;
			const result = await listUsers({ limit: 1, offset: 0 });
			usersTotal = result.total || result.users.length;
			usersError = null;
		} catch (err) {
			if (err instanceof Error && err.message.includes("401")) {
				usersTotal = null;
			}
			usersError = err instanceof Error ? err : new Error(String(err));
		} finally {
			usersLoading = false;
		}
	}

	async function fetchHealth() {
		// En Tauri, asumimos que está bien
		if (isTauri()) {
			healthData = { status: "ok" };
			healthLoading = false;
			return;
		}
		
		try {
			healthLoading = true;
			const response = await fetch("http://localhost:3000/health");
			if (!response.ok) throw new Error("API unhealthy");
			healthData = await response.json();
			healthError = null;
		} catch (err) {
			healthError = err instanceof Error ? err : new Error(String(err));
		} finally {
			healthLoading = false;
		}
	}

	onMount(() => {
		fetchUsers();
		fetchHealth();

		const interval = setInterval(fetchHealth, 30000);
		return () => clearInterval(interval);
	});
</script>

<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
	<KpiCard
		title={m.kpi_total_users()}
		value={usersTotal ?? "—"}
		badge={{ text: m.database(), variant: "secondary" }}
		change={{ value: 12, label: "+12%" }}
	/>

	<KpiCard
		title={m.kpi_api_health()}
		value={healthData ? m.kpi_status_ok() : healthLoading ? m.kpi_status_checking() : m.kpi_status_error()}
		badge={{
			text: healthData ? m.kpi_status_online() : healthError ? m.kpi_status_offline() : m.kpi_status_checking(),
			variant: healthData ? "default" : healthError ? "destructive" : "outline"
		}}
	/>

	<KpiCard
		title={m.kpi_active_sessions()}
		value="N/A"
		badge={{ text: m.kpi_coming_soon(), variant: "outline" }}
	/>

	<KpiCard
		title={m.kpi_database()}
		value="SQLite"
		badge={{ text: m.kpi_local(), variant: "secondary" }}
		change={{ value: 12, label: "~12ms", type: "neutral" }}
	/>
</div>

{#if usersError}
	<div class="mt-4 p-4 border border-red-200 bg-red-50 rounded text-red-700">
		{m.error_loading()}: {usersError.message}
	</div>
{/if}
