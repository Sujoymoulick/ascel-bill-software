<script lang="ts">
  import { onMount } from 'svelte';
  import { isTauri } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { api, dateLabel, defaultPrintSettings, errorText, money, stateLabel, today } from './data';
  import type { Business, Currency, Draft, Invoice, Item, OverviewData, RegisterRow, StateOption } from './data';
  import logo from '../src-tauri/icons/brand.png';
  import Setup from './Setup.svelte';
  import Records from './Records.svelte';
  import InvoiceEditor from './InvoiceEditor.svelte';
  import InvoicePrint from './InvoicePrint.svelte';
  import InvoiceSettings from './InvoiceSettings.svelte';
  import Overview from './Overview.svelte';
  import StockAdjust from './StockAdjust.svelte';
  import { formatCount, languageOptions, locale, setLocale, translator } from './locale';
  import type { Language } from './locale';
  import './app-copy';

  const sections = [
    { name: 'Sales', pages: [['overview', 'Overview'], ['invoice', 'New invoice'], ['register', 'Invoices']] },
    { name: 'Business', pages: [['customers', 'Customers'], ['items', 'Items']] },
    { name: 'Reports and settings', pages: [['reports', 'Sales register'], ['data', 'Backup & restore'], ['settings', 'Settings']] },
  ] as const;
  type Page = typeof sections[number]['pages'][number][0];
  type RecordsMode = 'add' | 'attention';
  let t = $derived(translator($locale));

  let page = $state<Page>('overview');
  let business = $state<Business | null>(null);
  let printSettings = $derived(business?.printSettings || defaultPrintSettings);
  let states = $state<StateOption[]>([]);
  let launching = $state(true);
  let startupError = $state('');
  let notice = $state('');
  let noticeValues = $state<Record<string, string | number>>({});
  let error = $state('');
  let loadError = $state('');
  let rows = $state<RegisterRow[]>([]);
  let metrics = $state<OverviewData | null>(null);
  let registerStatus = $state('issued');
  let registerQuery = $state('');
  let registerPage = $state(0);
  let reportMonth = $state(today().slice(0, 7));
  let reportCurrency = $state<Currency>('INR');
  let editorSeed = $state<Draft | null>(null);
  let pendingDraft = $state<Draft | null>(null);
  let editorKey = $state(0);
  let viewed = $state<Invoice | null>(null);
  let busy = $state(false);
  let loadingRows = $state(false);
  let restorePath = $state<string | null>(null);
  let stockItem = $state<Item | null>(null);
  let stockRevision = $state(0);
  let recordsMode = $state<RecordsMode | null>(null);
  let loadRequest = 0;

  onMount(() => {
    let disposed = false;
    let release: (() => void) | undefined;
    void startup();
    if (isTauri()) {
      void listen('workspace-changed', () => { if (page === 'overview' && business) void overview(); })
        .then((unlisten) => { if (disposed) unlisten(); else release = unlisten; })
        .catch(() => { /* Navigation still refreshes the local workspace. */ });
    }
    return () => { disposed = true; release?.(); };
  });

  async function startup() {
    launching = true;
    startupError = '';
    try {
      if (!isTauri()) throw new Error('Open the Ascel Bill desktop application to access local data.');
      [business, states] = await Promise.all([api.workspace(), api.states()]);
      if (business) go('overview');
    } catch (cause) {
      startupError = errorText(cause);
    } finally {
      launching = false;
    }
  }

  function go(target: Page, mode: RecordsMode | null = null) {
    if (page === 'invoice' && target === 'invoice') return;
    page = target;
    recordsMode = mode;
    error = '';
    loadError = '';
    notice = '';
    noticeValues = {};
    restorePath = null;
    loadRequest += 1;
    window.scrollTo(0, 0);
    if (target === 'overview') { metrics = null; rows = []; void overview(); }
    if (target === 'register') { registerPage = 0; rows = []; void loadRegister(); }
    if (target === 'invoice') {
      editorSeed = pendingDraft;
      pendingDraft = null;
      editorKey += 1;
    }
  }

  function ready(workspace: Business) {
    business = workspace;
    go('overview');
  }

  async function overview() {
    const key = ++loadRequest;
    loadError = '';
    if (!metrics) loadingRows = true;
    try {
      const [stats, latest] = await Promise.all([api.overview(), api.register()]);
      if (key !== loadRequest) return;
      metrics = stats;
      rows = latest.slice(0, 6);
    } catch (cause) {
      if (key === loadRequest) loadError = errorText(cause);
    } finally {
      if (key === loadRequest) loadingRows = false;
    }
  }

  function resumeDrafts() {
    registerStatus = 'draft';
    registerQuery = '';
    go('register');
  }

  async function openStock(id: string) {
    error = '';
    try {
      const [item] = await api.chosenItems([id]);
      if (item) stockItem = item;
    } catch (cause) {
      error = errorText(cause);
    }
  }

  function stockSaved(item: Item) {
    stockRevision += 1;
    notice = 'Stock updated for {item}.';
    noticeValues = { item: item.name };
  }

  async function loadRegister() {
    const key = ++loadRequest;
    loadError = '';
    loadingRows = true;
    try {
      const result = await api.register(registerStatus, registerQuery, registerPage);
      if (key === loadRequest) rows = result;
    } catch (cause) {
      if (key === loadRequest) loadError = errorText(cause);
    } finally {
      if (key === loadRequest) loadingRows = false;
    }
  }

  async function showRow(row: RegisterRow) {
    const targetPage = page;
    try {
      if (row.number) {
        const document = await api.invoice(row.id);
        if (page === targetPage) viewed = document;
      } else {
        const draft = await api.draft(row.id);
        if (page === targetPage) {
          pendingDraft = draft;
          go('invoice');
        }
      }
    } catch (cause) {
      if (page === targetPage) error = errorText(cause);
    }
  }

  async function perform(task: 'export' | 'backup' | 'choose' | 'restore') {
    if (busy) return;
    busy = true;
    error = '';
    notice = '';
    try {
      if (task === 'export') {
        const result = await api.exportSales(reportMonth, reportCurrency, t('Export sales register'));
        if (result !== null) {
          noticeValues = { count: formatCount(result, $locale) };
          notice = result === 1 ? '{count} invoice exported.' : '{count} invoices exported.';
        }
      } else if (task === 'backup') {
        const path = await api.backup(t('Save Ascel Bill backup'));
        if (path) notice = 'Backup saved.';
      } else if (task === 'choose') {
        const path = await api.pickBackup(t('Choose Ascel Bill backup'));
        if (typeof path === 'string') restorePath = path;
      } else if (restorePath) {
        await api.restore(restorePath);
        const restored = await api.workspace();
        if (restored) ready(restored);
        notice = 'Backup restored.';
      }
    } catch (cause) {
      error = errorText(cause);
    } finally {
      busy = false;
    }
  }

  function issued(document: Invoice) {
    registerStatus = 'issued';
    registerQuery = '';
    go('register');
    viewed = document;
  }

  async function updateBusiness(event: SubmitEvent) {
    event.preventDefault();
    if (busy || !business) return;
    busy = true;
    error = '';
    notice = '';
    const fields = new FormData(event.currentTarget as HTMLFormElement);
    try {
      business = await api.updateBusiness(String(fields.get('name') || '').trim(),
        String(fields.get('address') || '').trim(), fields.get('irn') === 'yes');
      notice = 'Business details saved.';
    } catch (cause) {
      error = errorText(cause);
    } finally {
      busy = false;
    }
  }

  async function updateCurrency(event: SubmitEvent) {
    event.preventDefault();
    if (busy || !business) return;
    const fields = new FormData(event.currentTarget as HTMLFormElement);
    busy = true;
    error = '';
    notice = '';
    try {
      business = await api.setUsdOption(fields.has('usd'));
      notice = 'Currency settings saved.';
    } catch (cause) {
      error = errorText(cause);
    } finally { busy = false; }
  }
</script>

<svelte:window onfocus={() => { if (page === 'overview' && metrics && metrics.endDate !== today()) void overview(); }}/>

{#if launching || startupError}
  <main class="setup-screen">
    <section class="setup-card startup">
      <img src={logo} alt="" width="40" height="40">
      <h1>Ascel Bill</h1>
      {#if startupError}<p class="error" role="alert">{t(startupError)}</p><button onclick={startup}>{t('Retry')}</button>
      {:else}<p role="status">{t('Opening workspace…')}</p>{/if}
    </section>
  </main>
{:else if !business && page !== 'data'}
  <Setup {states} onready={ready} onrestore={() => go('data')}/>
{:else}
  <div class="app-shell">
    <a class="skip-link" href="#workspace">{t('Skip to workspace')}</a>
    <aside class="sidebar">
      <div class="brand"><img src={logo} alt="" width="29" height="29"><strong>Ascel Bill</strong></div>
      {#if business}
        <div class="business-name"><strong title={business.name}>{business.name}</strong><small>{t(stateLabel(business.state, states))}</small></div>
        <nav aria-label={t('Workspace')}>
          {#each sections as group}
            <div class="nav-group">
              <p class="nav-label">{t(group.name)}</p>
              {#each group.pages as [id, title]}
                <button type="button" aria-current={page === id ? 'page' : undefined} class:nav-new={id === 'invoice'}
                  onclick={() => go(id)} disabled={busy}>{#if id === 'invoice'}<span aria-hidden="true">+</span>{/if}{t(title)}</button>
              {/each}
            </div>
          {/each}
        </nav>
      {:else}
        <nav aria-label={t('Setup')}><button onclick={() => go('overview')} disabled={busy}>{t('Return to setup')}</button></nav>
      {/if}
      <p class="local-footer"><span aria-hidden="true"></span>{t('Works offline')}</p>
    </aside>

    <main id="workspace" tabindex="-1">
      <div class="workspace-tools">
        <time datetime={today()}>{dateLabel(today(), $locale)}</time>
        <label class="language-switch"><span class="sr-only">{t('Interface language')}</span>
          <select value={$locale} onchange={(event) => setLocale(event.currentTarget.value as Language)}>
            {#each languageOptions as option}<option value={option.value}>{option.label}</option>{/each}
          </select>
        </label>
      </div>
      {#if error}<p class="error" role="alert">{t(error)}</p>{/if}
      {#if loadError}
        <div class="error load-error"><p role="alert">{t(loadError)}</p>
          <button type="button" onclick={page === 'overview' ? overview : loadRegister}>{t('Retry')}</button>
        </div>
      {/if}
      {#if notice}<p class="success" role="status">{t(notice, noticeValues)}</p>{/if}

      {#if page === 'overview' && business}
        <Overview data={metrics} loading={loadingRows} canViewUsd={!!business.allowUsd}
          onnew={() => go('invoice')} oncustomers={(mode) => go('customers', mode)} onitems={(mode) => go('items', mode)} onstock={openStock}/>
        <section class="panel">
          <div class="panel-header"><h2>{t('Recent invoices')}</h2>
            <div class="actions">
              <button onclick={resumeDrafts}>{t('Resume drafts')}</button>
              <button onclick={() => { registerStatus = 'issued'; registerQuery = ''; go('register'); }}>{t('View all')}</button>
            </div>
          </div>
          {@render registerTable()}
        </section>
      {:else if page === 'invoice' && business}
        {#key editorKey}
          <InvoiceEditor {business} {states} initial={editorSeed} onissued={issued} ondata={(kind, draft) => { pendingDraft = draft; go(kind); }}/>
        {/key}
      {:else if page === 'register' && business}
        <header class="page-head"><h1>{t('Invoices')}</h1><button class="primary" onclick={() => { pendingDraft = null; go('invoice'); }}>{t('New invoice')}</button></header>
        <section class="panel">
          <form class="toolbar" onsubmit={(event) => { event.preventDefault(); registerPage = 0; void loadRegister(); }}>
            <label class="inline-label">{t('Status')}
              <select bind:value={registerStatus} onchange={(event) => { registerStatus = event.currentTarget.value; registerPage = 0; registerQuery = ''; void loadRegister(); }}>
                <option value="issued">{t('Issued')}</option><option value="draft">{t('Drafts')}</option>
              </select>
            </label>
            {#if registerStatus === 'issued'}
              <input type="search" bind:value={registerQuery} aria-label={t('Search invoices')} placeholder={t('Invoice number or customer')}>
              <button type="submit" disabled={loadingRows}>{t('Search')}</button>
            {/if}
          </form>
          {@render registerTable()}
          {#if registerPage > 0 || rows.length === 60}
            <div class="toolbar pagination">
              <button onclick={() => { registerPage -= 1; void loadRegister(); }} disabled={registerPage === 0 || loadingRows}>{t('Previous')}</button>
              <span>{t('Page')} {formatCount(registerPage + 1, $locale)}</span>
              <button onclick={() => { registerPage += 1; void loadRegister(); }} disabled={rows.length < 60 || loadingRows}>{t('Next')}</button>
            </div>
          {/if}
        </section>
      {:else if (page === 'items' || page === 'customers') && business}
        {#key page}<Records kind={page} initialMode={recordsMode} {business} {states} onstock={(item) => stockItem = item} {stockRevision}/>{/key}
      {:else if page === 'reports' && business}
        <header class="page-head"><h1>{t('Sales register')}</h1></header>
        <form class="panel entry-form compact" onsubmit={(event) => { event.preventDefault(); void perform('export'); }}>
          <h2>{t('Export issued invoices')}</h2>
          <p class="help">{t('CSV for your accountant. Includes taxable value and GST by component.')}</p>
          <div class="fields">
            <label>{t('Month')} <input type="month" bind:value={reportMonth} required></label>
            <label>{t('Invoice currency')}
              <select bind:value={reportCurrency}><option value="INR">INR · ₹</option><option value="USD">USD · $</option></select>
            </label>
          </div>
          <div class="actions"><button class="primary" type="submit" disabled={busy}>{t(busy ? 'Exporting…' : 'Export CSV')}</button></div>
        </form>
      {:else if page === 'settings' && business}
        <header class="page-head"><h1>{t('Settings')}</h1></header>
        <div class="settings-layout">
          <div>
            <form class="panel entry-form" onsubmit={updateBusiness}>
              <h2>{t('Business and GST')}</h2>
              <label>{t('Business name')} <input name="name" value={business.name} maxlength="100" required></label>
              <label>{t('Address for invoices')} <textarea name="address" value={business.address} rows="3" maxlength="350" required></textarea></label>
              <p class="help">{t(stateLabel(business.state, states))} · {business.gstin || t('Unregistered')} · {t('Series')} {business.prefix}</p>
              {#if business.gstin}
                <label>{t('E-invoicing for B2B sales')}
                  <select name="irn" value={business.irnRequired ? 'yes' : 'no'}>
                    <option value="yes">{t('Keep B2B invoices in draft pending IRN')}</option>
                    <option value="no">{t('E-invoicing is not required, confirmed')}</option>
                  </select>
                </label>
              {/if}
              <div class="actions"><button class="primary" disabled={busy}>{t(busy ? 'Saving…' : 'Save business settings')}</button></div>
            </form>
            <section class="panel entry-form">
              <h2>{t('Interface language')}</h2>
              <label><span class="sr-only">{t('Interface language')}</span>
                <select value={$locale} onchange={(event) => setLocale(event.currentTarget.value as Language)}>
                  {#each languageOptions as option}<option value={option.value}>{option.label}</option>{/each}
                </select>
              </label>
              <p class="help">{t('Language applies to this device and to invoice previews.')}</p>
            </section>
            <form class="panel entry-form" onsubmit={updateCurrency}>
              <h2>{t('Invoice currency')}</h2>
              <p class="help">{t('New invoices start in INR.')}</p>
              {#if business.gstin}
                <p class="help">{t('GST invoices use INR in this app.')}</p>
              {:else}
                <label class="check-field"><input type="checkbox" name="usd" checked={business.allowUsd}>{t('Offer USD on ordinary invoices')}</label>
                <p class="help">{t('Enter USD prices directly on the invoice or in the item catalogue.')}</p>
                <div class="actions"><button class="primary" disabled={busy}>{t(busy ? 'Saving…' : 'Save currency settings')}</button></div>
              {/if}
            </form>
          </div>
          <InvoiceSettings settings={printSettings} onSaved={(updated) => { business = updated; }}/>
        </div>
      {:else if page === 'data'}
        <header class="page-head"><h1>{t('Backup & restore')}</h1></header>
        <div class="backup-grid">
          {#if business}
            <section class="panel entry-form"><h2>{t('Create backup')}</h2>
              <p class="help">{t('Save a copy on a separate drive. The file contains business records.')}</p>
              <button class="primary" disabled={busy} onclick={() => perform('backup')}>{t('Save backup')}</button>
            </section>
          {/if}
          <section class="panel entry-form"><h2>{t('Restore a backup')}</h2>
            <p class="help">{t('Restore an Ascel Bill backup on this workstation.')}</p>
            <button onclick={() => perform('choose')} disabled={busy}>{t('Choose backup')}</button>
          </section>
        </div>
        {#if restorePath}
          <section class="panel entry-form confirm">
            <h2>{t('Restore this workspace?')}</h2>
            <p class="help">{t('Restore replaces the local workspace and keeps a recovery copy.')}</p>
            <p class="file-name">{restorePath}</p>
            <div class="actions"><button onclick={() => restorePath = null} disabled={busy}>{t('Cancel')}</button>
              <button class="primary" onclick={() => perform('restore')} disabled={busy}>{t(busy ? 'Restoring…' : 'Restore backup')}</button></div>
          </section>
        {/if}
      {/if}
    </main>
  </div>
{/if}

{#if viewed}<InvoicePrint invoice={viewed} {states} settings={printSettings} onclose={() => viewed = null}/>{/if}
{#if stockItem}<StockAdjust item={stockItem} onclose={() => stockItem = null} onsaved={stockSaved}/>{/if}

{#snippet registerTable()}
  {#if loadingRows}
    <p class="empty" role="status">{t('Loading invoices…')}</p>
  {:else if rows.length}
    <div class="table-wrap"><table>
      <thead><tr><th scope="col">{t('Invoice')}</th><th scope="col">{t('Customer')}</th><th scope="col">{t('Date')}</th><th scope="col">{t('Payment')}</th><th scope="col" class="numeric">{t('Total')}</th><th scope="col"><span class="sr-only">{t('Action')}</span></th></tr></thead>
      <tbody>{#each rows as row (row.id)}
        <tr><td><strong>{row.number || t('Draft')}</strong></td><td>{row.customer === 'Walk-in customer' ? t('Walk-in customer') : row.customer}</td><td>{dateLabel(row.date, $locale)}</td>
          <td>{t(row.payment)}</td><td class="numeric">{row.totals ? money(row.totals.total, row.currency, $locale) : '—'}</td>
          <td><button onclick={() => showRow(row)}>{t(row.number ? 'View' : 'Open draft')}</button></td></tr>
      {/each}</tbody>
    </table></div>
  {:else if !loadError}
    <p class="empty">{t(page === 'register' && registerStatus === 'draft' ? 'No saved drafts.' : 'No issued invoices found.')}</p>
  {/if}
{/snippet}
