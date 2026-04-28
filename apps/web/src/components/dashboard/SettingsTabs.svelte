<script lang="ts">
	import * as Card from "$lib/components/ui/card/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
	import SettingsManager from "../admin/SettingsManager.svelte";
	import { authStore } from "$lib/stores/auth.svelte";
	import * as m from "$lib/paraglide/messages.js";

	let activeTab = $state<'profile' | 'system'>('profile');

	const isAdmin = $derived(authStore.user?.permissions?.includes('admin:read') ?? false);

	function switchTab(tab: 'profile' | 'system') {
		activeTab = tab;
	}
</script>

<div class="space-y-6">
	<!-- Header -->
	<div>
		<h1 class="text-3xl font-bold tracking-tight">{m.settings_title()}</h1>
		<p class="text-muted-foreground mt-2">
			{m.settings_manage()}
		</p>
	</div>

	<!-- Tab Navigation -->
	<div class="flex gap-1 border-b border-slate-800">
		<button
			type="button"
			class="px-4 py-2 text-sm font-medium transition-colors relative
				{activeTab === 'profile' ? 'text-white' : 'text-slate-400 hover:text-white'}"
			onclick={() => switchTab('profile')}
		>
			{m.settings_tab_profile()}
			{#if activeTab === 'profile'}
				<span class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary"></span>
			{/if}
		</button>

		{#if isAdmin}
			<button
				type="button"
				class="px-4 py-2 text-sm font-medium transition-colors relative
					{activeTab === 'system' ? 'text-white' : 'text-slate-400 hover:text-white'}"
				onclick={() => switchTab('system')}
			>
				{m.settings_tab_system()}
				{#if activeTab === 'system'}
					<span class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary"></span>
				{/if}
			</button>
		{/if}
	</div>

	<!-- Profile Tab -->
	{#if activeTab === 'profile'}
		<Card.Root>
			<Card.Header>
				<Card.Title>{m.settings_profile_info()}</Card.Title>
				<Card.Description>
					{m.settings_profile_desc()}
				</Card.Description>
			</Card.Header>
			<Card.Content>
				<form class="space-y-4">
					<div class="grid gap-2">
						<Label for="name">{m.label_name()}</Label>
						<Input id="name" type="text" value={authStore.user?.name ?? ''} />
					</div>
					<div class="grid gap-2">
						<Label for="email">{m.label_email()}</Label>
						<Input id="email" type="email" value={authStore.user?.email ?? ''} />
					</div>
				</form>
			</Card.Content>
			<Card.Footer>
				<Button>{m.settings_save_changes()}</Button>
			</Card.Footer>
		</Card.Root>

		<Card.Root>
			<Card.Header>
				<Card.Title>{m.settings_security()}</Card.Title>
				<Card.Description>
					{m.settings_security_desc()}
				</Card.Description>
			</Card.Header>
			<Card.Content>
				<div class="space-y-4">
					<div class="flex items-center justify-between">
						<div>
							<p class="font-medium">{m.settings_2fa()}</p>
							<p class="text-sm text-muted-foreground">{m.settings_2fa_desc()}</p>
						</div>
						<Button variant="outline" size="sm">{m.settings_enable()}</Button>
					</div>
					<div class="flex items-center justify-between">
						<div>
							<p class="font-medium">{m.settings_change_password()}</p>
							<p class="text-sm text-muted-foreground">{m.settings_change_password_desc()}</p>
						</div>
						<Button variant="outline" size="sm">{m.settings_change()}</Button>
					</div>
				</div>
			</Card.Content>
		</Card.Root>
	{/if}

	<!-- System Tab (solo admins) -->
	{#if activeTab === 'system' && isAdmin}
		<Card.Root class="bg-slate-900 border-slate-800">
			<Card.Header>
				<Card.Title>{m.settings_system_config()}</Card.Title>
				<Card.Description>
					{m.settings_system_desc()}
				</Card.Description>
			</Card.Header>
			<Card.Content>
				<SettingsManager />
			</Card.Content>
		</Card.Root>
	{/if}
</div>