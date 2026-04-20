// Runtime type definitions for Paraglide

export function setLanguageTag(tag: string): void;
export function getLanguageTag(): string;
export const availableLanguageTags: readonly string[];
export const defaultLanguageTag: string;
export function isAvailableLanguageTag(tag: string): boolean;
export function detectLanguageTag(request: Request | Headers): string;
