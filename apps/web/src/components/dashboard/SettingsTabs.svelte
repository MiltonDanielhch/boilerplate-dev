<script lang="ts">
	import * as Card from "$lib/components/ui/card/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
	import SettingsManager from "../admin/SettingsManager.svelte";
	import { authStore } from "$lib/stores/auth.svelte";

	let activeTab = $state<'profile' | 'system'>('profile');

	const isAdmin = $derived(authStore.user?.permissions?.includes('admin:read') ?? false);

	function switchTab(tab: 'profile' | 'system') {
		activeTab = tab;
	}
</script>

<div class="space-y-6">
	<!-- Header -->
	<div>
		<h1 class="text-3xl font-bold tracking-tight">Settings</h1>
		<p class="text-muted-foreground mt-2">
			Manage your account and system configuration
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
			Profile
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
				System
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
				<Card.Title>Profile Information</Card.Title>
				<Card.Description>
					Update your personal information and profile details
				</Card.Description>
			</Card.Header>
			<Card.Content>
				<form class="space-y-4">
					<div class="grid gap-2">
						<Label for="name">Name</Label>
						<Input id="name" type="text" placeholder="Your name" value={authStore.user?.name ?? ''} />
					</div>
					<div class="grid gap-2">
						<Label for="email">Email</Label>
						<Input id="email" type="email" placeholder="your@email.com" value={authStore.user?.email ?? ''} />
					</div>
				</form>
			</Card.Content>
			<Card.Footer>
				<Button>Save Changes</Button>
			</Card.Footer>
		</Card.Root>

		<Card.Root>
			<Card.Header>
				<Card.Title>Security</Card.Title>
				<Card.Description>
					Update your security preferences
				</Card.Description>
			</Card.Header>
			<Card.Content>
				<div class="space-y-4">
					<div class="flex items-center justify-between">
						<div>
							<p class="font-medium">Two-Factor Authentication</p>
							<p class="text-sm text-muted-foreground">Add an extra layer of security</p>
						</div>
						<Button variant="outline" size="sm">Enable</Button>
					</div>
					<div class="flex items-center justify-between">
						<div>
							<p class="font-medium">Change Password</p>
							<p class="text-sm text-muted-foreground">Update your account password</p>
						</div>
						<Button variant="outline" size="sm">Change</Button>
					</div>
				</div>
			</Card.Content>
		</Card.Root>
	{/if}

	<!-- System Tab (solo admins) -->
	{#if activeTab === 'system' && isAdmin}
		<Card.Root class="bg-slate-900 border-slate-800">
			<Card.Header>
				<Card.Title>System Configuration</Card.Title>
				<Card.Description>
					Configure global application parameters, features, and maintenance
				</Card.Description>
			</Card.Header>
			<Card.Content>
				<SettingsManager />
			</Card.Content>
		</Card.Root>
	{/if}
</div>