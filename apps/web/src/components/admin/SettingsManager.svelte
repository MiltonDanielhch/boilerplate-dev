<script lang="ts">
	import { onMount } from "svelte";
	import * as Card from "$lib/components/ui/card/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
	import { Switch } from "$lib/components/ui/switch/index.js";
	import { RefreshCw, Save, Settings, ShieldAlert, Globe } from "lucide-svelte";
	import api from "$lib/api/axios";

	interface Setting {
		key: string;
		value: string;
		description: string | null;
	}

	let settings = $state<Setting[]>([]);
	let loading = $state(true);
	let saving = $state<string | null>(null);

	async function loadSettings() {
		loading = true;
		try {
			const res = await api.get("/admin/settings");
			settings = res.data;
		} catch (err) {
			console.error("Failed to load settings", err);
		} finally {
			loading = false;
		}
	}

	async function updateSetting(key: string, value: string) {
		saving = key;
		try {
			await api.put(`/admin/settings/${key}`, { value });
		} catch (err) {
			alert(`Failed to update ${key}`);
		} finally {
			saving = null;
		}
	}

	onMount(() => loadSettings());

	// Helpers for boolean toggles
	function isBoolean(val: string) {
		return val === 'true' || val === 'false';
	}
</script>

<div class="max-w-3xl space-y-6">
	{#if loading}
		<div class="space-y-4">
			{#each Array(3) as _}
				<div class="h-24 bg-slate-900/50 animate-pulse rounded-lg border border-slate-800"></div>
			{/each}
		</div>
	{:else}
		{#each settings as setting}
			<Card.Root class="bg-slate-900 border-slate-800">
				<Card.Content class="pt-6">
					<div class="flex items-center justify-between gap-6">
						<div class="space-y-1 flex-1">
							<div class="flex items-center gap-2">
								<Label class="text-sm font-bold uppercase tracking-wider text-slate-200">
									{setting.key.replace(/_/g, ' ')}
								</Label>
								{#if setting.key.includes('maintenance') || setting.key.includes('allow')}
									<ShieldAlert class="h-3 w-3 text-amber-500" />
								{/if}
							</div>
							<p class="text-xs text-slate-500">{setting.description ?? 'No description provided.'}</p>
						</div>

						<div class="flex items-center gap-4">
							{#if isBoolean(setting.value)}
								<Switch 
									checked={setting.value === 'true'} 
									onCheckedChange={(checked) => updateSetting(setting.key, checked ? 'true' : 'false')}
									disabled={saving === setting.key}
								/>
							{:else}
								<div class="flex gap-2">
									<Input 
										bind:value={setting.value} 
										class="w-48 bg-slate-950 border-slate-800 h-9" 
									/>
									<Button 
										size="sm" 
										variant="secondary"
										onclick={() => updateSetting(setting.key, setting.value)}
										disabled={saving === setting.key}
									>
										{saving === setting.key ? '...' : 'Save'}
									</Button>
								</div>
							{/if}
						</div>
					</div>
				</Card.Content>
			</Card.Root>
		{/each}
	{/if}

	<div class="pt-6 border-t border-slate-800">
		<Button variant="outline" onclick={loadSettings}>
			<RefreshCw class="h-4 w-4 mr-2" />
			Reload Settings
		</Button>
	</div>
</div>
