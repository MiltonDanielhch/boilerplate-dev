// Paraglide runtime
// Provides runtime utilities for i18n

/**
 * Current language tag
 * @type {string}
 */
let currentLanguageTag = "es";

/**
 * Set the language tag
 * @param {string} tag
 */
export function setLanguageTag(tag) {
	currentLanguageTag = tag;
}

/**
 * Get the current language tag
 * @returns {string}
 */
export function getLanguageTag() {
	return currentLanguageTag;
}

/**
 * Available language tags
 * @type {readonly string[]}
 */
export const availableLanguageTags = ["es", "en"];

/**
 * Default language tag
 * @type {string}
 */
export const defaultLanguageTag = "es";

/**
 * Check if a language tag is available
 * @param {string} tag
 * @returns {boolean}
 */
export function isAvailableLanguageTag(tag) {
	return availableLanguageTags.includes(tag);
}

/**
 * Detect language from request/headers
 * @param {Request | Headers} request
 * @returns {string}
 */
export function detectLanguageTag(request) {
	// Simple detection - default to Spanish
	return "es";
}
