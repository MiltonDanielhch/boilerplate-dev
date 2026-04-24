// Ubicación: `apps/web/src/lib/tauri.ts`
//
// Descripción: Utilidades para la integración con Tauri 2.0.
//              Permite detectar si estamos en entorno de escritorio y
//              comunicarse con el backend de Rust.
//
// ADRs relacionados: 0030 (Multiplataforma Tridente)

/**
 * Detecta si la aplicación se está ejecutando dentro de Tauri.
 */
export const isTauri = (): boolean => {
	return typeof window !== "undefined" && (window as any).__TAURI_INTERNALS__ !== undefined;
};

/**
 * Wrapper para invoke que falla silenciosamente o maneja errores si no estamos en Tauri.
 */
export async function tauriInvoke<T>(command: string, args: Record<string, any> = {}): Promise<T> {
	if (!isTauri()) {
		throw new Error(`Intentando llamar al comando Tauri "${command}" fuera de entorno desktop.`);
	}
	
	const { invoke } = await import("@tauri-apps/api/core");
	return await invoke<T>(command, args);
}
