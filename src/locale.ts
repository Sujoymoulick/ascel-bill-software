import { writable } from 'svelte/store';
import { messages } from './locale-catalog';

export type Language = 'en' | 'hi' | 'bn';
export type TranslationValues = Record<string, string | number>;
export const languageOptions = [
  { value: 'en', label: 'English' },
  { value: 'hi', label: 'हिन्दी' },
  { value: 'bn', label: 'বাংলা' },
] as const;

const storageKey = 'ascel-bill.language';

function isLanguage(value: unknown): value is Language {
  return value === 'en' || value === 'hi' || value === 'bn';
}

function savedLanguage(): Language {
  try {
    const value = globalThis.localStorage?.getItem(storageKey);
    if (isLanguage(value)) return value;
  } catch {
    // The first launch still works when webview storage is unavailable.
  }
  return 'en';
}

export const locale = writable<Language>(savedLanguage());

locale.subscribe((language) => {
  if (globalThis.document) globalThis.document.documentElement.lang = language;
  try { globalThis.localStorage?.setItem(storageKey, language); }
  catch { /* The current session keeps the chosen language. */ }
});

export function setLocale(language: unknown) {
  if (isLanguage(language)) locale.set(language);
}

export function registerTranslations(catalog: Record<string, readonly [string, string]>) {
  Object.assign(messages, catalog);
}

export const localeTag = (language: Language = 'en') => ({ en: 'en-IN', hi: 'hi-IN', bn: 'bn-IN' })[language];

const countFormatters = Object.fromEntries(languageOptions.map(({ value }) => [value, new Intl.NumberFormat(localeTag(value))])) as Record<Language, Intl.NumberFormat>;
export function formatCount(value: number, language: Language = 'en') {
  return countFormatters[language].format(value);
}

const quantityFormatters = Object.fromEntries(languageOptions.map(({ value }) =>
  [value, new Intl.NumberFormat(localeTag(value), { maximumFractionDigits: 3 })])) as Record<Language, Intl.NumberFormat>;
export function formatQuantity(milli: number, language: Language = 'en') {
  return quantityFormatters[language].format(milli / 1_000);
}

const dynamicMessages: readonly [RegExp, string, string][] = [
  [/^Add an HSN for (.+) before issuing a GST invoice\.$/, 'Add an HSN for {name} before issuing a GST invoice.', 'name'],
  [/^The discount on (.+) exceeds its line amount\.$/, 'The discount on {name} exceeds its line amount.', 'name'],
  [/^This phone or email is already saved for (.+)\.$/, 'This phone or email is already saved for {name}.', 'name'],
  [/^This phone or email is saved for (.+)\. Select the existing customer or correct the details\.$/, 'This phone or email is saved for {name}. Select the existing customer or correct the details.', 'name'],
  [/^Insufficient stock for (.+)\. The draft has been kept\.$/, 'Insufficient stock for {name}. The draft has been kept.', 'name'],
  [/^Local data error:\s*([\s\S]+)$/, 'Local data error: {detail}', 'detail'],
  [/^Could not open printing:\s*([\s\S]+)$/, 'Could not open printing: {detail}', 'detail'],
];

export function translate(language: Language, english: string, values?: TranslationValues): string {
  const localized = language !== 'en' && Object.prototype.hasOwnProperty.call(messages, english)
    ? messages[english][language === 'hi' ? 0 : 1]
    : english;
  if (localized === english && language !== 'en' && !values) {
    for (const [pattern, key, parameter] of dynamicMessages) {
      const match = english.match(pattern);
      if (match) return translate(language, key, { [parameter]: match[1] });
    }
  }
  return values ? localized.replace(/\{([A-Za-z][A-Za-z0-9]*)\}/g, (marker, key) =>
    Object.prototype.hasOwnProperty.call(values, key) ? String(values[key]) : marker) : localized;
}

export function translator(language: Language) {
  return (english: string, values?: TranslationValues) => translate(language, english, values);
}
