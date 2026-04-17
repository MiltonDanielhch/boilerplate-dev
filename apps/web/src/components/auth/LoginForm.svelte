<!--
  Ubicación: `apps/web/src/components/auth/LoginForm.svelte`

  Descripción: Formulario de login con validación ArkType,
               integración con auth store y manejo de errores.

  ADRs relacionados: 0022 (Frontend), 0008 (PASETO), 0021 (OpenAPI)
-->

<script lang="ts">
	import { type } from "arktype";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
	import * as Card from "$lib/components/ui/card/index.js";
	import { authStore } from "$lib/stores/auth.svelte";
	import { login } from "$lib/api/auth";

	// Schema de validación inline (ArkType)
	const LoginSchema = type({
		email: "string.email",
		password: "string >= 8"
	});

	// Estado del formulario
	let email = $state("");
	let password = $state("");
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);

	// Validación reactiva
	let validation = $derived(LoginSchema({ email, password }));
	let isValid = $derived(!validation.errors);

	async function handleSubmit(e: Event) {
		e.preventDefault();
		errorMessage = null;

		const result = LoginSchema({ email, password });
		if (result.errors) {
			errorMessage = "Please check your inputs";
			return;
		}

		isSubmitting = true;
		try {
			await login({ email, password });
			// Redirect to dashboard on success
			window.location.href = "/dashboard";
		} catch (err) {
			errorMessage = err instanceof Error ? err.message : "Login failed";
		} finally {
			isSubmitting = false;
		}
	}
</script>

<Card.Root>
	<Card.Content class="pt-6">
		<form onsubmit={handleSubmit} class="space-y-4">
			<div class="space-y-2">
				<Label for="email">Email</Label>
				<Input
					id="email"
					type="email"
					placeholder="you@example.com"
					bind:value={email}
					disabled={isSubmitting}
				/>
			</div>

			<div class="space-y-2">
				<Label for="password">Password</Label>
				<Input
					id="password"
					type="password"
					placeholder="••••••••"
					bind:value={password}
					disabled={isSubmitting}
				/>
			</div>

			{#if errorMessage}
				<div class="text-sm text-red-600 bg-red-50 p-2 rounded">
					{errorMessage}
				</div>
			{/if}

			<Button type="submit" class="w-full" disabled={isSubmitting || !isValid}>
				{isSubmitting ? "Signing in..." : "Sign in"}
			</Button>
		</form>
	</Card.Content>
</Card.Root>
