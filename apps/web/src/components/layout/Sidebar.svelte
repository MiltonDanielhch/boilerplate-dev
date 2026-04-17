<!--
  Ubicación: `apps/web/src/components/layout/Sidebar.svelte`

  Descripción: Componente de navegación lateral colapsable usando Svelte 5 Runes.
               Incluye menú de navegación con iconos de lucide-svelte,
               estado colapsable con $state y eventos custom para
               sincronización con el layout principal.

  ADRs relacionados: 0022 (Frontend), 0023 (i18n - labels traducibles)
-->

<script lang="ts">
	import { cn } from "$lib/utils.js";
	import Button from "$lib/components/ui/button/button.svelte";
	import { ChevronLeft, ChevronRight, LayoutDashboard, Users, Settings, LogOut } from "lucide-svelte";

	let { class: className }: { class?: string } = $props();
	
	let collapsed = $state(false);
	
	const menuItems = [
		{ icon: LayoutDashboard, label: "Dashboard", href: "/dashboard" },
		{ icon: Users, label: "Users", href: "/users" },
		{ icon: Settings, label: "Settings", href: "/settings" },
	];
	
	function toggleSidebar() {
		collapsed = !collapsed;
		dispatchSidebarToggle(collapsed);
	}
	
	function dispatchSidebarToggle(isCollapsed: boolean) {
		if (typeof document !== 'undefined') {
			document.dispatchEvent(new CustomEvent('sidebar-toggle', {
				detail: { collapsed: isCollapsed }
			}));
		}
	}
</script>

<aside
	class={cn(
		"fixed left-0 top-0 z-40 h-screen border-r bg-sidebar transition-all duration-300",
		collapsed ? "w-16" : "w-64",
		className
	)}
>
	<div class="flex h-full flex-col">
		<!-- Header -->
		<div class="flex h-16 items-center justify-between border-b px-4">
			{#if !collapsed}
				<span class="font-semibold text-sidebar-foreground">🚀 Boilerplate</span>
			{/if}
			<Button variant="ghost" size="icon" onclick={toggleSidebar} class="ml-auto">
				{#if collapsed}
					<ChevronRight class="h-4 w-4" />
				{:else}
					<ChevronLeft class="h-4 w-4" />
				{/if}
			</Button>
		</div>
		
		<!-- Navigation -->
		<nav class="flex-1 space-y-1 p-2">
			{#each menuItems as item}
				{#if collapsed}
					<a
						href={item.href}
						class="flex h-10 w-full items-center justify-center rounded-md hover:bg-sidebar-accent hover:text-sidebar-accent-foreground transition-colors"
						title={item.label}
					>
						<item.icon class="h-5 w-5" />
					</a>
				{:else}
					<Button
						variant="ghost"
						class="w-full justify-start gap-3"
						href={item.href}
					>
						<item.icon class="h-5 w-5" />
						{item.label}
					</Button>
				{/if}
			{/each}
		</nav>
		
		<!-- Footer -->
		<div class="border-t p-2">
			{#if collapsed}
				<button
					class="flex h-10 w-full items-center justify-center rounded-md hover:bg-sidebar-accent hover:text-sidebar-accent-foreground transition-colors"
					title="Logout"
				>
					<LogOut class="h-5 w-5" />
				</button>
			{:else}
				<Button variant="ghost" class="w-full justify-start gap-3">
					<LogOut class="h-5 w-5" />
					Logout
				</Button>
			{/if}
		</div>
	</div>
</aside>
