/**
 * Ubicación: `apps/web/src/lib/i18n/formatters.ts`
 *
 * Descripción: Formatters para internacionalización (i18n).
 *              Formatea fechas, monedas (BOB), y números según locale.
 *              Timezone por defecto: America/La_Paz (Bolivia).
 *
 * ADRs relacionados: 0022 (Frontend), 0023 (i18n)
 */

// Opciones por defecto para Bolivia
const DEFAULT_LOCALE = "es-BO";
const DEFAULT_TIMEZONE = "America/La_Paz";
const DEFAULT_CURRENCY = "BOB"; // Boliviano

/**
 * Formatea una fecha ISO a formato localizado.
 *
 * @param isoString - Fecha en formato ISO (ej: "2026-04-20T12:00:00Z")
 * @param locale - Locale (default: "es-BO")
 * @param options - Opciones de Intl.DateTimeFormat
 * @returns Fecha formateada (ej: "20/04/2026")
 */
export function formatDate(
	isoString: string | Date,
	locale: string = DEFAULT_LOCALE,
	options: Intl.DateTimeFormatOptions = {}
): string {
	const date = typeof isoString === "string" ? new Date(isoString) : isoString;

	const defaultOptions: Intl.DateTimeFormatOptions = {
		year: "numeric",
		month: "2-digit",
		day: "2-digit",
		timeZone: DEFAULT_TIMEZONE,
		...options,
	};

	return new Intl.DateTimeFormat(locale, defaultOptions).format(date);
}

/**
 * Formatea fecha y hora juntas.
 *
 * @param isoString - Fecha en formato ISO
 * @param locale - Locale (default: "es-BO")
 * @returns Fecha y hora formateada (ej: "20/04/2026, 08:00")
 */
export function formatDateTime(
	isoString: string | Date,
	locale: string = DEFAULT_LOCALE
): string {
	return formatDate(isoString, locale, {
		year: "numeric",
		month: "2-digit",
		day: "2-digit",
		hour: "2-digit",
		minute: "2-digit",
		hour12: false,
	});
}

/**
 * Formatea hora.
 *
 * @param isoString - Fecha en formato ISO
 * @param locale - Locale (default: "es-BO")
 * @returns Hora formateada (ej: "08:00")
 */
export function formatTime(
	isoString: string | Date,
	locale: string = DEFAULT_LOCALE
): string {
	const date = typeof isoString === "string" ? new Date(isoString) : isoString;

	return new Intl.DateTimeFormat(locale, {
		hour: "2-digit",
		minute: "2-digit",
		hour12: false,
		timeZone: DEFAULT_TIMEZONE,
	}).format(date);
}

/**
 * Formatea un monto en moneda (Boliviano BOB por defecto).
 *
 * @param amount - Monto numérico
 * @param locale - Locale (default: "es-BO")
 * @param currency - Código de moneda (default: "BOB")
 * @returns Monto formateado (ej: "Bs 1.234,56")
 */
export function formatCurrency(
	amount: number,
	locale: string = DEFAULT_LOCALE,
	currency: string = DEFAULT_CURRENCY
): string {
	return new Intl.NumberFormat(locale, {
		style: "currency",
		currency,
		minimumFractionDigits: 2,
		maximumFractionDigits: 2,
	}).format(amount);
}

/**
 * Formatea un número con separadores locales.
 *
 * @param n - Número a formatear
 * @param locale - Locale (default: "es-BO")
 * @param decimals - Cantidad de decimales (default: 0)
 * @returns Número formateado (ej: "1.234.567,89")
 */
export function formatNumber(
	n: number,
	locale: string = DEFAULT_LOCALE,
	decimals: number = 0
): string {
	return new Intl.NumberFormat(locale, {
		minimumFractionDigits: decimals,
		maximumFractionDigits: decimals,
	}).format(n);
}

/**
 * Formatea un porcentaje.
 *
 * @param n - Valor entre 0 y 1 (ej: 0.25 = 25%)
 * @param locale - Locale (default: "es-BO")
 * @returns Porcentaje formateado (ej: "25%")
 */
export function formatPercent(
	n: number,
	locale: string = DEFAULT_LOCALE
): string {
	return new Intl.NumberFormat(locale, {
		style: "percent",
		minimumFractionDigits: 0,
		maximumFractionDigits: 1,
	}).format(n);
}

/**
 * Formatea una fecha relativa ("hace 2 días", "hace 1 hora").
 *
 * @param isoString - Fecha en formato ISO
 * @param locale - Locale (default: "es")
 * @returns Texto relativo
 */
export function formatRelativeTime(
	isoString: string | Date,
	locale: string = "es"
): string {
	const date = typeof isoString === "string" ? new Date(isoString) : isoString;
	const now = new Date();
	const diffMs = now.getTime() - date.getTime();
	const diffSecs = Math.floor(diffMs / 1000);
	const diffMins = Math.floor(diffSecs / 60);
	const diffHours = Math.floor(diffMins / 60);
	const diffDays = Math.floor(diffHours / 24);

	const rtf = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });

	if (diffDays > 0) {
		return rtf.format(-diffDays, "day");
	}
	if (diffHours > 0) {
		return rtf.format(-diffHours, "hour");
	}
	if (diffMins > 0) {
		return rtf.format(-diffMins, "minute");
	}
	return rtf.format(-diffSecs, "second");
}
