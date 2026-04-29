<!--
  Ubicación: `apps/web/src/components/layout/Sidebar.svelte`

  Descripción: Componente de navegación lateral colapsable usando Svelte 5 Runes.
               Incluye menú de navegación con iconos de lucide-svelte,
               estado colapsable con $state y RBAC para filtrar items
               según permisos del usuario.

  ADRs relacionados: 0022 (Frontend), 0023 (i18n), 0006 (RBAC)
-->

<script lang="ts">
	import { cn } from "$lib/utils.js";
	import Button from "$lib/components/ui/button/button.svelte";
	import { ChevronLeft, ChevronRight, LayoutDashboard, Users, Settings, LogOut, Shield, ClipboardList, MousePointerClick, FileEdit } from "lucide-svelte";
	import { authStore } from "$lib/stores/auth.svelte";
	import { logout } from "$lib/api/auth";
	import * as m from "$lib/paraglide/messages.js";
	import { isTauri } from "$lib/tauri";

	let { class: className }: { class?: string } = $props();

	let collapsed = $state(false);
	let mobileOpen = $state(false);
	
	// Estado local del usuario para reactividad
	let currentUser = $state(authStore.user);

	// Sincronizar cuando authStore.user cambie
	$effect(() => {
		// Este effect se re-ejecuta cuando authStore.user cambia
		currentUser = authStore.user;
	});

	// Cargar estado del sidebar desde localStorage
	$effect(() => {
		if (typeof localStorage !== 'undefined') {
			const saved = localStorage.getItem('sidebar_collapsed');
			if (saved !== null) {
				collapsed = saved === 'true';
			}
		}
	});

	// Persistir cambios en localStorage
	$effect(() => {
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem('sidebar_collapsed', String(collapsed));
		}
	});

	interface MenuItem {
		icon: typeof LayoutDashboard;
		label: string;
		href: string;
		permission?: string;
	}

	const allMenuItems: MenuItem[] = [
		{ icon: LayoutDashboard, label: m.sidebar_dashboard(), href: "/dashboard" },
		{ icon: Users, label: m.sidebar_users(), href: "/dashboard/users", permission: "users:read" },
		
		// Admin Section
		{ icon: MousePointerClick, label: m.sidebar_leads(), href: "/admin/leads", permission: "admin:read" },
		{ icon: FileEdit, label: m.sidebar_cms(), href: "/admin/content", permission: "admin:read" },
		{ icon: Settings, label: m.sidebar_settings(), href: "/dashboard/settings", permission: "admin:read" },
		{ icon: ClipboardList, label: m.sidebar_audit(), href: "/admin/audit", permission: "audit:read" },
		{ icon: Shield, label: m.sidebar_security(), href: "/admin/security", permission: "admin:read" },
		
		{ icon: Shield, label: m.sidebar_roles(), href: "/dashboard/roles", permission: "roles:read" },
	];

	// Filtrar items según permisos
	const menuItems = $derived(
		allMenuItems.filter(item => {
			if (!item.permission) return true;
			if (!currentUser) return true;
			return currentUser.permissions?.includes(item.permission) ?? false;
		})
	);
	
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

	// Listen for mobile toggle events
	$effect(() => {
		if (typeof document !== 'undefined') {
			const handler = () => {
				mobileOpen = !mobileOpen;
			};
			document.addEventListener('sidebar-mobile-toggle', handler);
			return () => document.removeEventListener('sidebar-mobile-toggle', handler);
		}
	});

	function closeMobile() {
		mobileOpen = false;
	}

	async function handleLogout() {
		try {
			await logout();
			window.location.href = '/login';
		} catch {
			// Si falla el logout en API, igual limpiamos local
			authStore.clearAuth();
			window.location.href = '/login';
		}
	}
</script>

<!-- Overlay para móvil -->
{#if mobileOpen}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div 
		class="fixed inset-0 z-40 bg-background/80 backdrop-blur-sm lg:hidden"
		onclick={closeMobile}
	></div>
{/if}

<aside
	class={cn(
		"fixed left-0 top-0 z-50 h-screen border-r bg-sidebar transition-all duration-300",
		"lg:translate-x-0", // Siempre visible en desktop
		mobileOpen ? "translate-x-0" : "-translate-x-full lg:translate-x-0", // Mostrar/ocultar en móvil
		collapsed ? "w-16" : "w-64",
		className
	)}
>
	<div class="flex h-full flex-col">
		<!-- Header -->
		<div class="flex h-16 items-center justify-between border-b px-4 gap-2">
			{#if !collapsed}
				<div class="flex flex-col min-w-0 flex-1">
					<span class="font-semibold text-sidebar-foreground truncate">🚀 {m.app_name()}</span>
					{#if isTauri()}
						<span class="text-[10px] font-bold uppercase tracking-wider text-primary">{m.desktop_mode()}</span>
					{/if}
				</div>
			{:else}
				<!-- Logo solo cuando colapsado -->
				<span class="font-bold text-sidebar-foreground text-lg">🚀</span>
			{/if}
			
			<!-- Botón toggle (desktop) -->
			<Button variant="ghost" size="icon" onclick={toggleSidebar} class="hidden lg:flex shrink-0">
				{#if collapsed}
					<ChevronRight class="h-4 w-4" />
				{:else}
					<ChevronLeft class="h-4 w-4" />
				{/if}
			</Button>
			
			<!-- Botón cerrar (móvil) -->
			<Button variant="ghost" size="icon" onclick={closeMobile} class="lg:hidden shrink-0">
				<ChevronLeft class="h-4 w-4" />
			</Button>
		</div>
		
		<!-- Navigation -->
		<nav class="flex-1 space-y-1 p-2 overflow-y-auto">
			{#each menuItems as item}
				{#if collapsed}
					<a
						href={item.href}
						class="flex h-10 w-full items-center justify-center rounded-md hover:bg-sidebar-accent hover:text-sidebar-accent-foreground transition-colors"
						title={item.label}
						onclick={closeMobile}
					>
						<item.icon class="h-5 w-5" />
					</a>
				{:else}
					<a
						href={item.href}
						class="flex h-9 w-full items-center gap-3 rounded-md px-3 py-2 text-sm hover:bg-sidebar-accent hover:text-sidebar-accent-foreground transition-colors"
						onclick={closeMobile}
					>
						<item.icon class="h-5 w-5" />
						{item.label}
					</a>
				{/if}
			{/each}
		</nav>
		
		<!-- Footer -->
		<div class="border-t p-2">
			{#if collapsed}
				<button
					class="flex h-10 w-full items-center justify-center rounded-md hover:bg-sidebar-accent hover:text-sidebar-accent-foreground transition-colors"
					title={m.logout_button()}
					onclick={handleLogout}
				>
					<LogOut class="h-5 w-5" />
				</button>
			{:else}
				<Button variant="ghost" class="w-full justify-start gap-3" onclick={handleLogout}>
					<LogOut class="h-5 w-5" />
					{m.logout_button()}
				</Button>
			{/if}
		</div>
	</div>
</aside>
