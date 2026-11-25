<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { formatQuantity, locale, translator } from './locale';
  import { api, currencySymbol, defaultPrintSettings, errorText, invoiceTemplates, money, newDraft, stateLabel } from './data';
  import type { Business, Currency, Customer, Draft, Invoice, InvoiceTemplate, Item, StateOption } from './data';
  import InvoicePrint from './InvoicePrint.svelte';

  type EditableLine = { item: Item; quantity: number | undefined; price: number | undefined; discount: number | undefined };
  let { business, states, initial, onissued, ondata }: {
    business: Business; states: StateOption[]; initial: Draft | null;
    onissued: (invoice: Invoice) => void; ondata: (kind: 'items' | 'customers', draft: Draft) => void;
  } = $props();

  let input = $state<Draft>(untrack(() => initial ?? newDraft(business)));
  let customer = $state<Customer | null>(null);
  let lineRows = $state<EditableLine[]>([]);
  let itemQuery = $state('');
  let customerQuery = $state('');
  let itemResults = $state<Item[]>([]);
  let customerResults = $state<Customer[]>([]);
  let picker = $state<'items' | 'customers' | null>(null);
  let opening = $state(true);
  let busy = $state(false);
  let error = $state('');
  let issueError = $state('');
  let status = $state('');
  let priceReminder = $state(false);
  let t = $derived(translator($locale));
  const payments = ['Credit', 'Cash', 'UPI', 'Bank transfer', 'Card'];
  let preview = $state<Invoice | null>(null);
  let contactMatch = $state<Customer | null>(null);
  let contactRequest = 0;
  let settings = $derived(business.printSettings || defaultPrintSettings);
  let currency = $derived(input.currency || 'INR');
  let usdAvailable = $derived(business.allowUsd && !business.gstin);
  let itemFilter: HTMLInputElement;
  let invoiceForm: HTMLFormElement;
  let searchRequest = 0;

  const estimate = $derived(lineRows.reduce((sum, line) => {
    const subtotal = Math.round((line.quantity || 0) * (line.price || 0) * 100);
    return sum + Math.max(0, subtotal - Math.round((line.discount || 0) * 100));
  }, 0));

  onMount(() => {
    void (async () => {
      try {
        const [items, selected] = await Promise.all([
          input.lines.length ? api.chosenItems(input.lines.map((line) => line.itemId)) : [],
          input.customerId ? api.chosenCustomer(input.customerId) : null,
        ]);
        customer = selected;
        lineRows = input.lines.map((line, index) => ({
          item: items[index],
          quantity: Number.isFinite(line.quantity) ? line.quantity / 1000 : undefined,
          price: Number.isFinite(line.price) ? line.price / 100 : undefined,
          discount: line.discount / 100,
        }));
      } catch (cause) {
        error = errorText(cause);
      } finally {
        opening = false;
      }
    })();
  });

  async function search(kind: 'items' | 'customers') {
    picker = kind;
    const request = ++searchRequest;
    try {
      if (kind === 'items') {
        const result = await api.items(itemQuery);
        if (searchRequest === request) itemResults = result;
      } else {
        const result = await api.customers(customerQuery);
        if (searchRequest === request) customerResults = result;
      }
    } catch (cause) {
      if (searchRequest === request) error = errorText(cause);
    }
  }

  function add(item: Item) {
    if (lineRows.length >= 100) {
      error = 'An invoice supports up to 100 items.';
      return;
    }
    const unitPrice = cataloguePrice(item, currency);
    lineRows.push({ item, quantity: 1, price: unitPrice == null ? undefined : unitPrice / 100, discount: 0 });
    itemQuery = '';
    picker = null;
    itemFilter.focus();
  }

  function cataloguePrice(item: Item, selected: Currency) {
    return selected === 'USD' ? item.usdPrice : item.price;
  }

  function changeCurrency(selected: Currency) {
    if (selected === currency || (selected === 'USD' && !usdAvailable)) return;
    input.currency = selected;
    for (const row of lineRows) {
      const unitPrice = cataloguePrice(row.item, selected);
      row.price = unitPrice == null ? undefined : unitPrice / 100;
      row.discount = 0;
    }
    priceReminder = lineRows.length > 0;
    status = '';
  }

  function selectCustomer(selected: Customer | null) {
    customer = selected;
    input.customerId = selected?.id || null;
    input.newCustomer = null;
    contactMatch = null;
    contactRequest += 1;
    input.place = selected?.state || business.state;
    customerQuery = '';
    picker = null;
  }

  function startCustomer() {
    const place = input.place;
    selectCustomer(null);
    input.place = place;
    input.newCustomer = { id: '', name: '', phone: '', email: '', gstin: '', address: '', state: place };
  }

  async function checkContact() {
    const request = ++contactRequest;
    contactMatch = null;
    const phone = input.newCustomer?.phone?.trim() || '';
    const email = input.newCustomer?.email?.trim() || '';
    if (!phone && !email) return;
    try {
      const match = await api.findContact(phone, email);
      if (request === contactRequest && (input.newCustomer?.phone?.trim() || '') === phone && (input.newCustomer?.email?.trim() || '') === email) {
        contactMatch = match;
      }
    } catch {
      // The saved draft and issue commands validate incomplete or conflicting contact details.
    }
  }

  function payload(): Draft {
    return { ...$state.snapshot(input), lines: lineRows.map((row) => ({
      itemId: row.item.id, quantity: Math.round(Number(row.quantity) * 1_000),
      price: Math.round(Number(row.price) * 100), discount: Math.round((row.discount || 0) * 100),
    })) };
  }

  async function save(review: boolean) {
    if (busy || opening || !invoiceForm.reportValidity()) return;
    if (review && !lineRows.length) {
      error = 'Add an item before reviewing the invoice.';
      itemFilter.focus();
      return;
    }
    error = '';
    status = '';
    picker = null;
    busy = true;
    try {
      const saved = await api.saveDraft(payload());
      input.id = saved.id;
      status = 'Draft saved.';
      if (review) {
        issueError = '';
        preview = await api.review(saved);
      }
    } catch (cause) {
      error = errorText(cause);
    } finally {
      busy = false;
    }
  }

  async function issue(template: InvoiceTemplate) {
    if (!input.id || busy) return;
    busy = true;
    issueError = '';
    try {
      const issued = await api.issue(input.id, template);
      preview = null;
      onissued(issued);
    } catch (cause) {
      issueError = errorText(cause);
    } finally {
      busy = false;
    }
  }

  function closePreview() {
    preview = null;
  }

  function keyboard(event: KeyboardEvent) {
    if (preview || opening || busy || event.repeat) return;
    if (event.key === 'F8') {
      event.preventDefault();
      itemFilter.focus();
    } else if (event.key === 'F9') {
      event.preventDefault();
      void save(true);
    }
  }
</script>

<svelte:window onkeydown={keyboard}/>
<header class="page-head">
  <div><h1>{t(input.id ? 'Invoice draft' : business.gstin ? 'New tax invoice' : 'New invoice')}</h1></div>
  <div class="actions">
    <button type="button" onclick={() => save(false)} disabled={busy || opening}>{t('Save draft')}</button>
    <button type="submit" form="invoice-form" class="primary" disabled={busy || opening}>{t('Review invoice')} <kbd>F9</kbd></button>
  </div>
</header>
{#if error}<p class="error" role="alert">{t(error)}</p>{/if}
{#if status}<p class="success" role="status">{t(status)}</p>{/if}
{#if priceReminder}<p class="panel-note" role="status">{t('Prices were reset for {currency}. Check prices and discounts before issuing.', { currency })}</p>{/if}
<form id="invoice-form" bind:this={invoiceForm} onsubmit={(event) => { event.preventDefault(); void save(true); }}>
  <div class="invoice-layout">
    <div class="invoice-main">
      <section class="panel">
        <div class="panel-header"><h2>{t('Invoice details')}</h2></div>
        <div class="panel-body fields">
           <div class="search-field">
            {#if input.newCustomer}
              <span class="customer-choice">{t('New customer')}</span>
              <button type="button" class="text-button selected-name" onclick={() => selectCustomer(null)}>{t('Use an existing customer or walk-in')}</button>
            {:else}
            <label>{t('Find customer')}
              <div class="search-control"><input value={customerQuery} placeholder={customer?.name || t('Walk-in customer')} oninput={(event) => { customerQuery = event.currentTarget.value; void search('customers'); }} maxlength="100">
                <button type="button" onclick={() => picker === 'customers' ? picker = null : search('customers')}>{t('Browse')}</button>
              </div>
            </label>
            {#if customer}
              <p class="help">{[customer.name, customer.phone, customer.email].filter(Boolean).join(' · ')}</p>
              <button type="button" class="text-button selected-name" onclick={() => selectCustomer(null)}>{t('Use walk-in instead')}</button>
            {:else}
              <button type="button" class="text-button selected-name" onclick={startCustomer}>{t('Add customer on invoice')}</button>
            {/if}
            {#if picker === 'customers'}
              <div class="lookup-list">
                {#each customerResults as party (party.id)}
                  <button type="button" onclick={() => selectCustomer(party)}><span>{party.name}</span><small>{party.phone || party.email || party.gstin || t(stateLabel(party.state, states))}</small></button>
                {:else}<span class="lookup-empty">{t('No customers found.')}</span>{/each}
                <button type="button" onclick={() => ondata('customers', payload())}>{t('Manage customers')}</button>
              </div>
            {/if}
            {/if}
          </div>
          <label>{t('Invoice date')} <input type="date" bind:value={input.date} required></label>
          {#if usdAvailable}
            <label>{t('Currency')}
              <select value={currency} disabled={opening || busy} onchange={(event) => changeCurrency(event.currentTarget.value as Currency)}>
                <option value="INR">{t('Indian rupee (INR)')}</option>
                <option value="USD">{t('US dollar (USD)')}</option>
              </select>
            </label>
          {:else if currency === 'USD'}
            <div>
              <p class="help">{t('USD is unavailable for this draft. Switch to INR.')}</p>
              <button type="button" disabled={opening || busy} onclick={() => changeCurrency('INR')}>{t('Switch to INR')}</button>
            </div>
          {/if}
          <label>{t('Place of supply')}
            <select bind:value={input.place} required>
              {#each states as [code, name]}<option value={code}>{code} · {t(name)}</option>{/each}
            </select>
          </label>
          <label>{t('Delivery address (optional)')} <input bind:value={input.deliveryAddress} maxlength="350"></label>
        </div>
        {#if input.newCustomer}
          {@const details = input.newCustomer}
          <div class="panel-body customer-fields">
            <p class="help">{t('The new customer is saved when the invoice is issued.')}</p>
            <div class="fields">
              <label>{t('Customer name')} <input bind:value={details.name} maxlength="100" required></label>
              <label>{t('Phone')} <input type="tel" bind:value={details.phone} maxlength="40" onblur={checkContact}></label>
              <label>{t('Email')} <input type="email" bind:value={details.email} maxlength="254" onblur={checkContact}></label>
              <label>{t('Customer state')}
                <select bind:value={details.state} onchange={(event) => { if (input.place === details.state) input.place = event.currentTarget.value; }}>
                  {#each states as [code, name]}<option value={code}>{t(name)}</option>{/each}
                </select>
              </label>
              <label>{t('GSTIN (optional)')} <input bind:value={details.gstin} maxlength="15" pattern={'[0-9A-Za-z]{15}'} autocapitalize="characters"></label>
              <label>{t('Billing address')} <textarea bind:value={details.address} maxlength="350" rows="2" required={!!details.gstin}></textarea></label>
            </div>
            {#if contactMatch}
              <div class="contact-match"><span>{t('Saved contact')} · <strong>{contactMatch.name}</strong></span><button type="button" onclick={() => selectCustomer(contactMatch)}>{t('Use saved customer')}</button></div>
            {/if}
          </div>
        {/if}
      </section>
      <section class="panel">
        <div class="panel-header"><h2>{t('Items')}</h2></div>
        <div class="panel-body search-field">
          <label>{t('Find item')}
            <div class="search-control"><input bind:this={itemFilter} value={itemQuery} placeholder={t('Search name or HSN')} oninput={(event) => { itemQuery = event.currentTarget.value; void search('items'); }} maxlength="100">
              <button type="button" onclick={() => picker === 'items' ? picker = null : search('items')}>{t('Browse')} <kbd>F8</kbd></button>
            </div>
          </label>
          {#if picker === 'items'}
            <div class="lookup-list">
              {#each itemResults as item (item.id)}
                {@const unitPrice = cataloguePrice(item, currency)}
                <button type="button" onclick={() => add(item)}><span>{item.name}</span><small>{unitPrice == null ? t('Enter a USD price') : money(unitPrice, currency, $locale)}{item.stock === null ? '' : ` · ${t('{count} in stock', { count: formatQuantity(item.stock, $locale) })}`}</small></button>
              {:else}<span class="lookup-empty">{t('No items found.')}</span>{/each}
              <button type="button" onclick={() => ondata('items', payload())}>{t('Manage items')}</button>
            </div>
          {/if}
        </div>
        {#if lineRows.length}
          <div class="table-wrap">
            <table class="line-table"><thead><tr><th scope="col">{t('Item')}</th><th scope="col">{t('Qty')}</th><th scope="col">{t('Rate {symbol}', { symbol: currencySymbol(currency) })}</th><th scope="col">{t('Discount {symbol}', { symbol: currencySymbol(currency) })}</th><th scope="col"><span class="sr-only">{t('Remove')}</span></th></tr></thead>
              <tbody>
                {#each lineRows as row, index (row)}
                  <tr><td><strong>{row.item.name}</strong><small>{row.item.hsn || row.item.unit}</small></td>
                    <td><input aria-label={t('Quantity for {name}', { name: row.item.name })} type="number" min="0.001" max="10000" step="0.001" bind:value={row.quantity} required></td>
                    <td><input aria-label={t('Rate for {name}', { name: row.item.name })} type="number" min="0" max="1000000" step="0.01" bind:value={row.price} required></td>
                    <td><input aria-label={t('Discount for {name}', { name: row.item.name })} type="number" min="0" step="0.01" bind:value={row.discount}></td>
                    <td><button type="button" class="remove" aria-label={t('Remove {name}', { name: row.item.name })} onclick={() => lineRows.splice(index, 1)}>×</button></td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {:else}<p class="empty">{t(opening ? 'Loading draft…' : 'Add the first item.')}</p>{/if}
      </section>
    </div>
    <div class="invoice-side">
      <section class="panel">
        <div class="panel-header"><h2>{t('Print layout')}</h2></div>
        <div class="panel-body">
          <label>{t('Invoice template')}
            <select value={input.template || settings.template} onchange={(event) => input.template = event.currentTarget.value as InvoiceTemplate}>
              {#each Object.entries(invoiceTemplates) as [key, template]}<option value={key}>{t(template.label)}</option>{/each}
            </select>
          </label>
        </div>
      </section>
      <section class="panel">
        <div class="panel-header"><h2>{t('Payment')}</h2></div>
        <div class="panel-body stacked">
          <label>{t('Method')}
            <select bind:value={input.payment}>{#each payments as value}<option {value}>{t(value)}</option>{/each}</select>
          </label>
          <label>{t('Reference (optional)')} <input bind:value={input.reference} maxlength="100"></label>
        </div>
      </section>
      <section class="panel">
        <div class="panel-header"><h2>{t(business.gstin ? 'Subtotal before GST' : 'Subtotal')}</h2></div>
        <div class="panel-body"><strong class="amount">{money(estimate, currency, $locale)}</strong>
          {#if business.gstin}<p class="help">{t(input.place === business.state ? 'Local supply. Review the invoice for exact GST totals.' : 'Interstate supply. Review the invoice for exact GST totals.')}</p>{/if}
        </div>
      </section>
    </div>
  </div>
</form>

{#if preview}<InvoicePrint invoice={preview} {states} {settings} {busy} error={issueError} onissue={issue}
  ontemplate={(template) => input.template = template} onclose={closePreview}/>{/if}
