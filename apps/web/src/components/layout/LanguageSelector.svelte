<!--
  Ubicación: `apps/web/src/components/layout/LanguageSelector.svelte`

  Descripción: Selector de idioma usando Paraglide JS.
               Guarda preferencia en localStorage y recarga la página.

  ADRs relacionados: 0023 (i18n)
-->

<script lang="ts">
	import { cn } from "$lib/utils.js";
	import * as m from "$lib/paraglide/messages.js";
	import { getLocale, setLocale, locales } from "$lib/paraglide/runtime.js";
	import { Languages, Check, ChevronDown } from "lucide-svelte";
	import Button from "$lib/components/ui/button/button.svelte";

	let { class: className }: { class?: string } = $props();

	// Estado del idioma actual
	let currentLocale = $state(getLocale());
	let isOpen = $state(false);

	// Sincronizar localStorage con cookie al cargar
	$effect(() => {
		if (typeof window === 'undefined') return;
		
		const storedLocale = localStorage.getItem('locale');
		if (storedLocale && (storedLocale === 'es' || storedLocale === 'en')) {
			// Actualizar cookie para que SSR use el mismo idioma
			document.cookie = `locale=${storedLocale};path=/;max-age=${60 * 60 * 24 * 365};SameSite=Lax`;
			
			// Si el idioma actual no coincide, cambiarlo
			if (storedLocale !== getLocale()) {
				setLocale(storedLocale as 'es' | 'en');
				currentLocale = storedLocale;
			}
		}
	});

	// Cerrar dropdown al hacer click fuera
	$effect(() => {
		if (!isOpen || typeof window === 'undefined') return;
		
		const handleClick = (e: MouseEvent) => {
			const target = e.target as HTMLElement;
			if (!target.closest('.language-selector')) {
				isOpen = false;
			}
		};
		
		window.addEventListener('click', handleClick);
		return () => window.removeEventListener('click', handleClick);
	});

	const languages = [
		{ code: 'es', label: 'Español', flag: '🇪🇸' },
		{ code: 'en', label: 'English', flag: '🇺🇸' },
	];

	function switchLanguage(code: string) {
		if (code === currentLocale || typeof window === 'undefined') return;
		
		isOpen = false;
		
		// Guardar preferencia en cookie (SSR-friendly) - 365 días
		document.cookie = `locale=${code};path=/;max-age=${60 * 60 * 24 * 365};SameSite=Lax`;
		
		// También en localStorage para backup
		localStorage.setItem('locale', code);
		
		// Cambiar idioma usando setLocale (recarga automáticamente)
		setLocale(code as 'es' | 'en');
	}

	function toggleOpen(e: Event) {
		e.stopPropagation();
		isOpen = !isOpen;
	}
</script>

<div class={cn("language-selector relative", className)}>
	<Button
		variant="ghost"
		size="sm"
		class="gap-2"
		onclick={toggleOpen}
	>
		<Languages class="h-4 w-4" />
		<span class="uppercase">{currentLocale}</span>
		<ChevronDown class="h-3 w-3" />
	</Button>

	{#if isOpen}
		<div class="absolute right-0 top-full mt-1 z-50 w-40 rounded-md border bg-popover p-1 shadow-md">
			<div class="px-2 py-1.5 text-sm font-semibold text-muted-foreground">
				{m.language()}
			</div>
			<div class="h-px bg-border my-1"></div>
			{#each languages as lang}
				<button
					class="w-full flex items-center justify-between px-2 py-2 text-sm rounded-sm hover:bg-accent hover:text-accent-foreground cursor-pointer"
					onclick={() => switchLanguage(lang.code)}
				>
					<span class="flex items-center gap-2">
						<span>{lang.flag}</span>
						<span>{lang.label}</span>
					</span>
					{#if currentLocale === lang.code}
						<Check class="h-4 w-4" />
					{/if}
				</button>
			{/each}
		</div>
	{/if}
</div>
