import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';

export type StateOption = [code: string, name: string];
export type Currency = 'INR' | 'USD';
export type StockMode = 'receive' | 'count';
type Language = 'en' | 'hi' | 'bn';
const localeTags = { en: 'en-IN', hi: 'hi-IN', bn: 'bn-IN' } as const;
export const invoiceTemplates = {
  standard: { label: 'A4 Standard', size: '210 × 297 mm' },
  ledger: { label: 'A4 Ledger', size: '210 × 297 mm' },
  'laser-a5': { label: 'A5 Laser', size: '148 × 210 mm' },
  'laser-a6': { label: 'A6 Laser', size: '105 × 148 mm' },
} as const;
export type InvoiceTemplate = keyof typeof invoiceTemplates;
export type PrintSettings = { template: InvoiceTemplate; accent: 'green' | 'navy' | 'black'; logo: string; footer: string;
  showContacts: boolean; showPaymentReference: boolean };
export const defaultPrintSettings: PrintSettings = { template: 'standard', accent: 'green', logo: '', footer: '', showContacts: false, showPaymentReference: true };
export type Business = { name: string; address: string; state: string; gstin: string; prefix: string; irnRequired: boolean; allowUsd?: boolean; printSettings?: PrintSettings };
export type Customer = { id: string; name: string; address: string; state: string; gstin: string; phone?: string; email?: string };
export type ImportSummary = { ready: number; imported: number; duplicates: number; invalid: number; issues: { line: number; message: string }[] };
export type Item = { id: string; name: string; hsn: string; unit: string; price: number; usdPrice?: number | null; rate: number; stock: number | null };
export type LineInput = { itemId: string; quantity: number; price: number; discount: number };
export type Draft = {
  id: string | null; date: string; customerId: string | null; place: string; deliveryAddress: string;
  payment: string; reference: string; lines: LineInput[]; newCustomer?: Customer | null; template?: InvoiceTemplate | null; currency?: Currency;
};
export type Totals = { taxable: number; discount: number; cgst: number; localGst: number; igst: number; total: number };
export type Invoice = {
  number: string | null; date: string; seller: Business; customer: Customer | null; place: string;
  deliveryAddress: string; payment: string; reference: string; localTaxName: string;
  lines: { name: string; hsn: string; unit: string; quantity: number; price: number; discount: number; rate: number;
    taxable: number; cgst: number; localGst: number; igst: number }[];
  totals: Totals; blockedReason: string | null; template?: InvoiceTemplate | null; printSettings?: PrintSettings; currency?: Currency;
};
export type RegisterRow = {
  id: string; number: string | null; date: string; customer: string; payment: string; totals: Totals | null; currency?: Currency;
};
export type Dashboard = { invoiceCount: number; sales: number; receivables: number; tax: number };
export type SalesAmounts = { invoiceCount: number; total: number; creditSales: number; tax: number };
export type DailySales = SalesAmounts & { date: string };
export type OverviewData = {
  periodDays: number; startDate: string; endDate: string;
  sales: Record<Currency, SalesAmounts & { days: DailySales[] }>;
  customerCount: number; activeCustomerCount: number;
  itemCount: number; trackedItemCount: number; inStockCount: number; lowStockCount: number;
  outOfStockCount: number; untrackedItemCount: number; reorderThreshold: number;
  stockAlerts: { id: string; name: string; unit: string; stock: number }[];
};

export const api = {
  workspace: () => invoke<Business | null>('workspace'),
  states: () => invoke<StateOption[]>('state_options'),
  saveBusiness: (business: Business) => invoke<Business>('save_business', { business }),
  updateBusiness: (name: string, address: string, irnRequired: boolean) => invoke<Business>('update_business', { name, address, irnRequired }),
  savePrintSettings: (settings: PrintSettings) => invoke<Business>('save_print_settings', { settings }),
  setUsdOption: (allowed: boolean) => invoke<Business>('set_usd_option', { allowed }),
  items: (query = '', needsAttention = false) => invoke<Item[]>('list_items', { query, needsAttention }),
  customers: (query = '') => invoke<Customer[]>('list_customers', { query }),
  chosenItems: (ids: string[]) => invoke<Item[]>('selected_items', { ids }),
  chosenCustomer: (id: string) => invoke<Customer>('selected_customer', { id }),
  saveItem: (item: Item) => invoke<Item>('save_item', { item }),
  updateStock: (id: string, mode: StockMode, quantity: number, note: string) => invoke<Item>('update_stock', { id, mode, quantity, note }),
  saveCustomer: (customer: Customer) => invoke<Customer>('save_customer', { customer }),
  findContact: (phone: string, email: string) => invoke<Customer | null>('find_customer_contact', { phone, email }),
  importCustomers: (content: string, apply = false) => invoke<ImportSummary>('import_customer_csv', { content, apply }),
  saveDraft: (input: Draft) => invoke<Draft>('save_draft', { input }),
  draft: (id: string) => invoke<Draft>('get_draft', { id }),
  review: (input: Draft) => invoke<Invoice>('review_invoice', { input }),
  issue: (id: string, template: InvoiceTemplate) => invoke<Invoice>('issue_invoice', { id, template }),
  invoice: (id: string) => invoke<Invoice>('get_invoice', { id }),
  register: (status = 'issued', query = '', page = 0) => invoke<RegisterRow[]>('invoice_register', { status, query, page }),
  dashboard: () => invoke<Dashboard>('dashboard'),
  overview: () => invoke<OverviewData>('overview', { asOf: today() }),
  printInvoice: (template: InvoiceTemplate) => invoke('print_invoice', { template }),
  exportSales: async (month: string, currency: Currency = 'INR', title = 'Export sales register') => {
    const suffix = currency === 'INR' ? '' : `-${currency}`;
    const path = await save({ title, defaultPath: `sales-${month}${suffix}.csv`, filters: [{ name: 'CSV', extensions: ['csv'] }] });
    return path ? invoke<number>('export_sales', { month, path, currency }) : null;
  },
  backup: async (title = 'Save Ascel Bill backup') => {
    const path = await save({ title, defaultPath: `Ascel-Bill-${today()}.ascelbill`, filters: [{ name: 'Ascel Bill backup', extensions: ['ascelbill'] }] });
    if (path) await invoke('backup_workspace', { path });
    return path;
  },
  pickBackup: (title = 'Choose Ascel Bill backup') => open({ title, multiple: false, filters: [{ name: 'Ascel Bill backup', extensions: ['ascelbill'] }] }),
  restore: (path: string) => invoke('restore_workspace', { path }),
};

const moneyFormats = new Map<string, Intl.NumberFormat>();
const dateFormats = new Map<Language, Intl.DateTimeFormat>();
export const currencySymbol = (currency: Currency = 'INR') => currency === 'USD' ? '$' : '₹';
export function money(minor: number, currency: Currency = 'INR', language: Language = 'en') {
  const key = `${language}:${currency}`;
  let formatter = moneyFormats.get(key);
  if (!formatter) {
    formatter = new Intl.NumberFormat(localeTags[language], { style: 'currency', currency, currencyDisplay: 'narrowSymbol' });
    moneyFormats.set(key, formatter);
  }
  return formatter.format(minor / 100);
}
export function dateLabel(date: string, language: Language = 'en') {
  let formatter = dateFormats.get(language);
  if (!formatter) {
    formatter = new Intl.DateTimeFormat(localeTags[language], { dateStyle: 'medium', timeZone: 'UTC' });
    dateFormats.set(language, formatter);
  }
  return formatter.format(new Date(`${date}T00:00:00Z`));
}
export const stateLabel = (code: string, states: StateOption[]) => states.find(([key]) => key === code)?.[1] || code;
export const errorText = (error: unknown) => error instanceof Error ? error.message : String(error);
export const qty = (milli: number, language: Language = 'en') => (milli / 1000).toLocaleString(localeTags[language], { maximumFractionDigits: 3 });

export function today() {
  const value = new Date();
  return [value.getFullYear(), String(value.getMonth() + 1).padStart(2, '0'), String(value.getDate()).padStart(2, '0')].join('-');
}

export function newDraft(workspace: Business): Draft {
  return { id: null, date: today(), customerId: null, place: workspace.state, deliveryAddress: '', payment: 'Credit', reference: '', lines: [], currency: 'INR' };
}

export function normalize(value: string) {
  return value.trim().toUpperCase();
}
