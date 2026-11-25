<script lang="ts">
  import { onMount, tick, untrack } from 'svelte';
  import { formatCount, formatQuantity, locale, translator } from './locale';
  import { api, errorText, money, normalize, stateLabel } from './data';
  import type { Business, Customer, Item, StateOption } from './data';
  import CustomerImport from './CustomerImport.svelte';

  let { kind, business, states, initialMode = null, onstock, stockRevision = 0 }: {
    kind: 'items' | 'customers'; business: Business; states: StateOption[];
    initialMode?: 'add' | 'attention' | null;
    onstock?: (item: Item) => void; stockRevision?: number;
  } = $props();
  let t = $derived(translator($locale));
  let rows = $state<(Item | Customer)[]>([]);
  let adding = $state(untrack(() => initialMode === 'add'));
  let editing = $state<Customer | null>(null);
  let importing = $state(false);
  let loading = $state(true);
  let saving = $state(false);
  let query = $state('');
  let needsAttention = $state(untrack(() => initialMode === 'attention'));
  let error = $state('');
  let listError = $state('');
  let status = $state('');
  let nameInput = $state<HTMLInputElement>();
  let attentionInput = $state<HTMLInputElement>();
  let request = 0;
  let title = $derived(kind === 'items' ? 'Items' : 'Customers');
  $effect(() => {
    // Mount the list once and refresh it after a stock adjustment.
    void stockRevision;
    untrack(() => { void list(); });
  });
  onMount(() => { (nameInput ?? (needsAttention ? attentionInput : undefined))?.focus(); });

  async function list() {
    const current = ++request;
    listError = '';
    loading = true;
    try {
      const found = kind === 'items' ? await api.items(query, needsAttention) : await api.customers(query);
      if (current === request) rows = found;
    } catch (cause) {
      if (current === request) listError = errorText(cause);
    } finally {
      if (current === request) loading = false;
    }
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (saving) return;
    saving = true;
    error = '';
    const data = new FormData(event.currentTarget as HTMLFormElement);
    const read = (key: string) => String(data.get(key) || '').trim();
    const name = read('name');
    try {
      if (kind === 'items') {
        const stock = read('stock');
        const usdPrice = read('usdPrice');
        await api.saveItem({ id: '', name, hsn: read('hsn'), unit: read('unit'), price: Math.round(Number(read('price')) * 100),
          rate: Math.round(Number(read('rate')) * 100), usdPrice: usdPrice === '' ? null : Math.round(Number(usdPrice) * 100), stock: stock === '' ? null : Math.round(Number(stock) * 1000) });
      } else {
        await api.saveCustomer({ id: editing?.id || '', name, address: read('address'), state: read('state'),
          gstin: normalize(read('gstin')), phone: read('phone'), email: read('email') });
      }
      adding = false;
      editing = null;
      query = '';
      needsAttention = false;
      status = name;
      await list();
    } catch (cause) {
      error = errorText(cause);
    } finally {
      saving = false;
    }
  }

  async function openEntry(customer: Customer | null = null) {
    editing = customer;
    importing = false;
    adding = true;
    error = '';
    status = '';
    await tick();
    nameInput?.focus();
  }
</script>

<header class="page-head">
  <h1>{t(title)}</h1>
  <div class="actions">
    {#if kind === 'customers'}<button type="button" onclick={() => { importing = true; adding = false; error = ''; status = ''; }}>{t('Import CSV')}</button>{/if}
    <button class="primary" type="button" onclick={() => openEntry()}>{t(kind === 'items' ? 'Add item' : 'Add customer')}</button>
  </div>
</header>
{#if error}<p class="error" role="alert">{t(error)}</p>{/if}
{#if status}<p class="success" role="status">{t('{name} saved.', { name: status })}</p>{/if}
{#if importing}<CustomerImport onClose={() => importing = false} onDone={() => { query = ''; void list(); }}/>{/if}
{#if adding}
  {#key editing?.id}
  <form class="panel entry-form" onsubmit={submit}>
    <h2>{t(kind === 'items' ? 'New item' : editing ? 'Edit customer' : 'New customer')}</h2>
    <div class="fields">
      <label>{t('Name')} <input name="name" bind:this={nameInput} value={editing?.name || ''} maxlength="100" required></label>
      {#if kind === 'items'}
        <label>HSN <input name="hsn" inputmode="numeric" maxlength="8" pattern={'[0-9]{4}|[0-9]{6}|[0-9]{8}'} required={!!business.gstin}></label>
        <label>{t('Unit')} <input name="unit" value="Nos" maxlength="12" required></label>
        <label>{t('Price excluding GST ({symbol})', { symbol: '₹' })} <input name="price" type="number" min="0" max="1000000" step="0.01" required></label>
        {#if business.allowUsd}<label>{t('Price in USD (optional)')} <input name="usdPrice" type="number" min="0" max="1000000" step="0.01"></label>{/if}
        <label>{t('GST rate (%)')} <input name="rate" type="number" min="0" max="100" step="0.01" value="0" required></label>
        <label>{t('Opening stock')} <input name="stock" type="number" min="0" max="1000000" step="0.001" placeholder={t('Leave blank if untracked')}></label>
      {:else}
        <label>{t('Phone')} <input name="phone" type="tel" value={editing?.phone || ''} maxlength="40"></label>
        <label>{t('Email')} <input name="email" type="email" value={editing?.email || ''} maxlength="254"></label>
        <label>{t('GSTIN (optional)')} <input name="gstin" value={editing?.gstin || ''} maxlength="15" pattern={'[0-9A-Za-z]{15}'} autocapitalize="characters"></label>
        <label>{t('State or union territory')}
          <select name="state">
            {#each states as [code, name]}<option value={code} selected={code === (editing?.state || business.state)}>{t(name)}</option>{/each}
          </select>
        </label>
        <label>{t('Address')} <textarea name="address" value={editing?.address || ''} maxlength="350" rows="2"></textarea></label>
      {/if}
    </div>
    <div class="actions">
      <button type="button" onclick={() => { adding = false; error = ''; }} disabled={saving}>{t('Cancel')}</button>
      <button class="primary" type="submit" disabled={saving}>{t(saving ? 'Saving…' : kind === 'items' ? 'Save item' : 'Save customer')}</button>
    </div>
  </form>
  {/key}
{/if}
<section class="panel" aria-busy={loading}>
  <form class="toolbar" onsubmit={(event) => { event.preventDefault(); void list(); }}>
    <input type="search" bind:value={query} aria-label={t(kind === 'items' ? 'Search items' : 'Search customers')} placeholder={t(kind === 'items' ? 'Search name or HSN' : 'Search name, phone, email or GSTIN')}>
    <button type="submit" disabled={loading}>{t('Search')}</button>
    {#if kind === 'items'}
      <label class="check-field"><input type="checkbox" bind:this={attentionInput} checked={needsAttention} onchange={(event) => {
        needsAttention = event.currentTarget.checked;
        void list();
      }}>{t('Low or out of stock only')}</label>
    {/if}
  </form>
  {#if listError}
    <div class="error load-error"><p role="alert">{t(listError)}</p><button type="button" onclick={list}>{t('Retry')}</button></div>
  {:else if rows.length}
    <div class="table-wrap">
      <table>
        <thead><tr><th scope="col">{t('Name')}</th>{#if kind === 'items'}<th scope="col">HSN</th><th scope="col" class="numeric">{business.allowUsd ? t('Price ({currency})', {currency: 'INR'}) : t('Price')}</th>{#if business.allowUsd}<th scope="col" class="numeric">{t('Price ({currency})', {currency: 'USD'})}</th>{/if}<th scope="col" class="numeric">GST</th><th scope="col" class="numeric">{t('In stock')}</th>{#if onstock}<th scope="col"><span class="sr-only">{t('Action')}</span></th>{/if}{:else}<th scope="col">{t('Contact')}</th><th scope="col">GSTIN</th><th scope="col">{t('State')}</th><th scope="col"><span class="sr-only">{t('Action')}</span></th>{/if}</tr></thead>
        <tbody>
          {#each rows as row (row.id)}
            <tr>
              <td><strong>{row.name}</strong></td>
              {#if 'hsn' in row}
                <td>{row.hsn || '—'}</td><td class="numeric">{money(row.price, 'INR', $locale)}</td>{#if business.allowUsd}<td class="numeric">{row.usdPrice == null ? '—' : money(row.usdPrice, 'USD', $locale)}</td>{/if}<td class="numeric">{formatCount(row.rate / 100, $locale)}%</td>
                <td class="numeric">{row.stock === null ? '—' : formatQuantity(row.stock, $locale)}<small>{row.unit}</small></td>
                {#if onstock}<td><button type="button" aria-label={t('Update stock for {name}', { name: row.name })} onclick={() => onstock?.(row)}>{t('Update stock')}</button></td>{/if}
              {:else}
                <td>{row.phone || (row.email ? '' : '—')}{#if row.email}<small>{row.email}</small>{/if}</td>
                <td>{row.gstin || '—'}</td><td>{t(stateLabel(row.state, states))}</td>
                <td><button type="button" aria-label={t('Edit {name}', { name: row.name })} onclick={() => openEntry(row)}>{t('Edit')}</button></td>
              {/if}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    {#if rows.length === 60}<p class="panel-note">{t('Showing the first 60 matches. Search for a name or code to narrow the results.')}</p>{/if}
  {:else}
    <p class="empty" role="status">{t(loading ? 'Loading…' : query ? 'No matches found.' : needsAttention ? 'No items are low or out of stock.' : kind === 'items' ? 'No items yet.' : 'No customers yet.')}</p>
  {/if}
</section>
