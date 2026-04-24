<script lang="ts">
	import { onMount } from "svelte";
	import { api } from "$lib/api/client";
	import * as Card from "$lib/components/ui/card/index.js";
	import { Server, Database, Clock, CheckCircle, XCircle, Loader } from "lucide-svelte";

	interface HealthStatus {
		status: string;
		database: string;
		timestamp: string;
	}

	let dbStatus = $state<"checking" | "ok" | "error">("checking");
	let apiStatus = $state<"checking" | "ok" | "error">("checking");
	let lastCheck = $state<string | null>(null);

	async function checkHealth() {
		dbStatus = "checking";
		apiStatus = "checking";

		try {
			const response = await api.get<HealthStatus>("/health");
			dbStatus = response.status === "ok" ? "ok" : "error";
			apiStatus = "ok";
			lastCheck = new Date().toLocaleTimeString();
		} catch {
			apiStatus = "error";
		}
	}

	onMount(() => {
		checkHealth();
		const interval = setInterval(checkHealth, 10000);
		return () => clearInterval(interval);
	});

	function getIcon(status: "checking" | "ok" | "error") {
		if (status === "checking") return Loader;
		if (status === "ok") return CheckCircle;
		return XCircle;
	}

	function getColor(status: "checking" | "ok" | "error"): string {
		if (status === "checking") return "text-yellow-500";
		if (status === "ok") return "text-green-500";
		return "text-red-500";
	}

	function getBgColor(status: "checking" | "ok" | "error"): string {
		if (status === "checking") return "bg-yellow-500/10";
		if (status === "ok") return "bg-green-500/10";
		return "bg-red-500/10";
	}
</script>

<Card.Root>
	<Card.Header>
		<Card.Title class="flex items-center gap-2">
			<Server class="h-5 w-5" />
			Estado del Sistema
		</Card.Title>
		<Card.Description>Monitoreo en tiempo real</Card.Description>
	</Card.Header>
	<Card.Content>
		<div class="space-y-4">
			<!-- API Status -->
			<div class="flex items-center gap-3">
				<div class="rounded-full p-2 {getBgColor(apiStatus)}">
					{#if apiStatus === 'checking'}
						<Loader class="h-4 w-4 {getColor(apiStatus)}" />
					{:else if apiStatus === 'ok'}
						<CheckCircle class="h-4 w-4 {getColor(apiStatus)}" />
					{:else}
						<XCircle class="h-4 w-4 {getColor(apiStatus)}" />
					{/if}
				</div>
				<div class="flex-1">
					<p class="text-sm font-medium">API</p>
					<p class="text-xs text-muted-foreground">
						{apiStatus === "checking"
							? "Verificando..."
							: apiStatus === "ok"
								? "En línea"
								: "Error de conexión"}
					</p>
				</div>
			</div>

			<!-- Database Status -->
			<div class="flex items-center gap-3">
				<div class="rounded-full p-2 {getBgColor(dbStatus)}">
					<Database class="h-4 w-4 {getColor(dbStatus)}" />
				</div>
				<div class="flex-1">
					<p class="text-sm font-medium">Base de Datos</p>
					<p class="text-xs text-muted-foreground">
						{dbStatus === "checking"
							? "Verificando..."
							: dbStatus === "ok"
								? "Conectada"
								: "Desconectada"}
					</p>
				</div>
			</div>

			<!-- Last Check -->
			{#if lastCheck}
				<div class="flex items-center gap-2 text-xs text-muted-foreground pt-2 border-t">
					<Clock class="h-3 w-3" />
					<span>Última verificación: {lastCheck}</span>
				</div>
			{/if}
		</div>
	</Card.Content>
</Card.Root>