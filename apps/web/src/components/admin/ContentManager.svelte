<script lang="ts">
	import { onMount } from "svelte";
	import * as Card from "$lib/components/ui/card/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Textarea } from "$lib/components/ui/textarea/index.js";
	import { Badge } from "$lib/components/ui/badge/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { FileText, Save, RefreshCw, Edit, Globe, Code } from "lucide-svelte";
	import api from "$lib/api/axios";

	interface ContentBlock {
		key: string;
		content: string;
		content_type: string;
		updated_at: string;
	}

	let blocks = $state<ContentBlock[]>([]);
	let loading = $state(true);
	let saving = $state<string | null>(null);

	async function loadBlocks() {
		loading = true;
		try {
			const res = await api.get("/admin/content");
			blocks = res.data;
		} catch (err) {
			console.error("Failed to load content blocks", err);
		} finally {
			loading = false;
		}
	}

	async function handleSave(key: string, content: string) {
		saving = key;
		try {
			await api.put(`/admin/content/${key}`, { content });
			// feedback visual opcional
		} catch (err) {
			alert("Failed to save content");
		} finally {
			saving = null;
		}
	}

	onMount(() => loadBlocks());
</script>

<div class="space-y-6">
	{#if loading}
		<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
			{#each Array(4) as _}
				<div class="h-48 bg-slate-900/50 animate-pulse rounded-lg border border-slate-800"></div>
			{/each}
		</div>
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
			{#each blocks as block}
				<Card.Root class="bg-slate-900 border-slate-800 flex flex-col">
					<Card.Header>
						<div class="flex items-center justify-between">
							<div class="flex items-center gap-2">
								<Globe class="h-4 w-4 text-primary" />
								<Card.Title class="text-sm font-mono">{block.key}</Card.Title>
							</div>
							<Badge variant="outline" class="text-[10px] uppercase border-slate-700">
								{block.content_type}
							</Badge>
						</div>
						<Card.Description class="text-xs">
							Last updated: {new Date(block.updated_at).toLocaleString()}
						</Card.Description>
					</Card.Header>
					<Card.Content class="flex-1">
						{#if block.content_type === 'text'}
							<Input 
								bind:value={block.content} 
								class="bg-slate-950 border-slate-800"
							/>
						{:else}
							<Textarea 
								bind:value={block.content} 
								rows={4}
								class="bg-slate-950 border-slate-800 font-mono text-xs"
							/>
						{/if}
					</Card.Content>
					<Card.Footer class="border-t border-slate-800 pt-4 flex justify-between items-center">
						<div class="text-[10px] text-slate-500 italic">
							{block.content.length} characters
						</div>
						<Button 
							size="sm" 
							onclick={() => handleSave(block.key, block.content)}
							disabled={saving === block.key}
						>
							{#if saving === block.key}
								<RefreshCw class="h-3 w-3 mr-2 animate-spin" />
								Saving...
							{:else}
								<Save class="h-3 w-3 mr-2" />
								Save
							{/if}
						</Button>
					</Card.Footer>
				</Card.Root>
			{/each}
		</div>
	{/if}

	<div class="flex justify-center pt-8">
		<Button variant="outline" onclick={loadBlocks}>
			<RefreshCw class="h-4 w-4 mr-2" />
			Refresh All Blocks
		</Button>
	</div>
</div>
