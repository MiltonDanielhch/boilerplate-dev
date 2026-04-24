<script lang="ts">
	import { onMount } from "svelte";
	import { api } from "$lib/api/client";
	import * as Card from "$lib/components/ui/card/index.js";
	import { formatDistanceToNow } from "date-fns";
	import { es } from "date-fns/locale";
	import { Activity, Clock, User, LogIn } from "lucide-svelte";

	interface AuditEntry {
		timestamp: string;
		method: string;
		uri: string;
		status: number;
		user_id: string | null;
	}

	let events = $state<AuditEntry[]>([]);
	let loading = $state(true);

	async function fetchRecentActivity() {
		loading = true;
		try {
			const response = await api.get<{ recent: AuditEntry[] }>("/audit/recent");
			events = response.recent || [];
		} catch {
			events = [];
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		fetchRecentActivity();
		const interval = setInterval(fetchRecentActivity, 30000);
		return () => clearInterval(interval);
	});

	function getMethodColor(method: string): string {
		const colors: Record<string, string> = {
			GET: "text-blue-500",
			POST: "text-green-500",
			PUT: "text-yellow-500",
			DELETE: "text-red-500",
			PATCH: "text-purple-500"
		};
		return colors[method] || "text-gray-500";
	}

	function getStatusColor(status: number): string {
		if (status >= 200 && status < 300) return "text-green-500";
		if (status >= 400 && status < 500) return "text-yellow-500";
		if (status >= 500) return "text-red-500";
		return "text-gray-500";
	}

	function formatTime(timestamp: string): string {
		try {
			return formatDistanceToNow(new Date(timestamp), { addSuffix: true, locale: es });
		} catch {
			return "reciente";
		}
	}

	function getIcon(method: string) {
		return method === "POST" || method === "PUT" ? LogIn : Activity;
	}
</script>

<Card.Root>
	<Card.Header>
		<Card.Title class="flex items-center gap-2">
			<Activity class="h-5 w-5" />
			Actividad Reciente
		</Card.Title>
		<Card.Description>Últimas acciones en el sistema</Card.Description>
	</Card.Header>
	<Card.Content>
		{#if loading}
			<div class="space-y-3">
				{#each Array(5) as _}
					<div class="flex gap-3 animate-pulse">
						<div class="h-8 w-8 rounded-full bg-muted"></div>
						<div class="flex-1 space-y-2">
							<div class="h-4 w-3/4 bg-muted rounded"></div>
							<div class="h-3 w-1/2 bg-muted rounded"></div>
						</div>
					</div>
				{/each}
			</div>
		{:else if events.length === 0}
			<div class="text-center py-8 text-muted-foreground">
				<p>No hay actividad reciente</p>
			</div>
		{:else}
			<div class="h-[300px] overflow-y-auto">
				<div class="space-y-3">
					{#each events as event (event.timestamp + event.uri)}
						{@const IconComponent = getIcon(event.method)}
						<div class="flex gap-3 items-start">
							<div class="rounded-full bg-primary/10 p-2">
								<IconComponent class="h-4 w-4" />
							</div>
							<div class="flex-1 min-w-0">
								<div class="flex items-center gap-2">
									<span class="font-mono text-sm {getMethodColor(event.method)}">{event.method}</span>
									<span class="text-sm truncate text-foreground">{event.uri}</span>
									<span class="text-sm {getStatusColor(event.status)}">({event.status})</span>
								</div>
								<div class="flex items-center gap-1 text-xs text-muted-foreground mt-1">
									<Clock class="h-3 w-3" />
									<span>{formatTime(event.timestamp)}</span>
									{#if event.user_id}
										<span class="flex items-center gap-1">
											<User class="h-3 w-3" />
											Usuario
										</span>
									{/if}
								</div>
							</div>
						</div>
					{/each}
				</div>
			</div>
		{/if}
	</Card.Content>
</Card.Root>