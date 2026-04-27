<!--
  Ubicación: `apps/web/src/components/auth/AuthInit.svelte`

  Descripción: Componente de inicialización de auth.
               Restaura sesión desde localStorage al cargar la app.

  ADRs relacionados: 0022 (Frontend), 0008 (PASETO)
-->

<script lang="ts">
	import { authStore } from "$lib/stores/auth.svelte";
	import { onMount } from "svelte";

	onMount(() => {
		authStore.initFromStorage();
	});
</script>

<!-- Este componente no renderiza nada visible -->
<slot />