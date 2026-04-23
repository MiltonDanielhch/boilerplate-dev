<script lang="ts">
// Ubicación: `apps/web/src/components/landing/LeadForm.svelte`
//
// Descripción: Formulario de captura de leads con validación.
//             Usa ArkType para validación en tiempo real.
//
// ADRs relacionados: 0022 (Frontend), 0029 (Landing + Leads), 0009 (Rate Limit)

import { createMutation } from "@tanstack/svelte-query";
import Button from "$lib/components/ui/button/button.svelte";
import { Input } from "$lib/components/ui/input/index.js";
import { apiClient } from "$lib/api/client";

type FormStatus = "idle" | "loading" | "success" | "error";

let email = $state("");
let name = $state("");
let honeypot = $state("");
let status = $state<FormStatus>("idle");
let errorMessage = $state("");

function validateEmail(e: string): boolean {
	const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
	return emailRegex.test(e);
}

async function handleSubmit(e: Event) {
	e.preventDefault();
	
	// Honeypot anti-spam
	if (honeypot) {
		status = "success";
		return;
	}
	
	// Validación cliente
	if (!validateEmail(email)) {
		status = "error";
		errorMessage = "Por favor ingresa un email válido";
		return;
	}
	
	status = "loading";
	
	try {
		const response = await apiClient.post("/api/v1/leads", {
			email,
			name: name || undefined
		});
		
		status = "success";
	} catch (error: any) {
		status = "error";
		if (error.status === 429) {
			errorMessage = "Demasiados intentos. Espera un momento.";
		} else if (error.status === 400) {
			errorMessage = "Email inválido";
		} else {
			errorMessage = "Algo falló. Intenta de nuevo.";
		}
	}
}
</script>

<form onsubmit={handleSubmit} class="flex flex-col gap-4 max-w-md mx-auto">
	<!-- Honeypot anti-spam -->
	<div class="hidden">
		<label for="website">Website</label>
		<input 
			type="text" 
			id="website" 
			name="website"
			bind:value={honeypot}
			tabindex="-1" 
			autocomplete="off"
		/>
	</div>
	
	{#if status === "success"}
		<div class="bg-green-500/10 border border-green-500/20 rounded-lg p-6 text-center">
			<p class="text-green-600 font-medium text-lg mb-2">
				¡Gracias! ✨
			</p>
			<p class="text-muted-foreground">
				Te avisaremos cuando lancemos.
			</p>
		</div>
	{:else}
		<div class="flex flex-col sm:flex-row gap-3">
			<div class="flex-1">
				<Input
					type="email"
					placeholder="tu@email.com"
					bind:value={email}
					required
					disabled={status === "loading"}
					class="w-full"
				/>
			</div>
			<Button type="submit" disabled={status === "loading"} class="w-full sm:w-auto">
				{#if status === "loading"}
					<span class="animate-spin">⏳</span>
				{:else}
					Notificarme
				{/if}
			</Button>
		</div>
		
		{#if status === "error"}
			<p class="text-destructive text-sm">{errorMessage}</p>
		{/if}
		
		<p class="text-xs text-muted-foreground text-center">
			No spam. Solo te avisaremos cuando lancemos.
		</p>
	{/if}
</form>