<script lang="ts">
  import { onMount, tick, untrack } from 'svelte';
  import { api, dateLabel, errorText, invoiceTemplates, money, stateLabel } from './data';
  import { formatCount, formatQuantity, locale, translator } from './locale';
  import type { Invoice, InvoiceTemplate, PrintSettings, StateOption } from './data';

  let { invoice, states, settings, busy = false, error = '', onclose, onissue, ontemplate }: {
    invoice: Invoice; states: StateOption[]; settings: PrintSettings; busy?: boolean; error?: string;
    onclose: () => void; onissue?: (template: InvoiceTemplate) => void; ontemplate?: (template: InvoiceTemplate) => void;
  } = $props();

  let dialog: HTMLDialogElement;
  let copy = $state('Recipient copy');
  let layoutSettings = $derived(invoice.printSettings ?? settings);
  let template = $state<InvoiceTemplate>(untrack(() => invoice.template ?? layoutSettings.template));
  let compact = $derived(template === 'laser-a5' || template === 'laser-a6');
  let printError = $state('');
  let t = $derived(translator($locale));
  const copies = ['Recipient copy', 'Transporter copy', 'Supplier copy'];
  onMount(() => { dialog.showModal(); });

  async function print() {
    printError = '';
    try { await tick(); await api.printInvoice(template); }
    catch (cause) { printError = errorText(cause); }
  }
</script>

<dialog bind:this={dialog} onclose={onclose} oncancel={(event) => { if (busy) event.preventDefault(); }} aria-labelledby="paper-title">
  <header class="dialog-head">
    <h2 id="paper-title">{t(invoice.number ? 'Issued invoice' : 'Review invoice')}</h2>
    <button type="button" onclick={() => dialog.close()} disabled={busy}>{t('Close')}</button>
  </header>
  <div class="print-options">
    <fieldset class="template-options" disabled={busy}>
      <legend>{t('Invoice template')}</legend>
      <div class="template-list">
        {#each Object.entries(invoiceTemplates) as [key, layout]}
          <label class="template-choice">
            <input type="radio" name="print-template" value={key} checked={template === key} onchange={() => {
              template = key as InvoiceTemplate;
              ontemplate?.(template);
            }}>
            <span>{t(layout.label)}</span>
          </label>
        {/each}
      </div>
    </fieldset>
    <p class="help">{invoiceTemplates[template].size}{invoice.number && compact ? ` · ${t('Use matching paper and print at actual size.')}` : ''}</p>
  </div>
  <article class={`invoice-paper paper-${template} accent-${layoutSettings.accent}`}>
    {@render paper()}
  </article>
  {#if invoice.blockedReason}<p class="error dialog-message" role="alert">{t(invoice.blockedReason)}</p>{/if}
  {#if error}<p class="error dialog-message" role="alert">{t(error)}</p>{/if}
  {#if printError}<p class="error dialog-message" role="alert">{t(printError)}</p>{/if}
  <footer class="dialog-actions">
    {#if invoice.number}
      <label>{t('Print copy')}
        <select bind:value={copy}>{#each copies as value}<option {value}>{t(value)}</option>{/each}</select>
      </label>
      <button type="button" class="primary" onclick={print}>{t('Print / Save as PDF')}</button>
    {:else if !invoice.blockedReason && onissue}
      <button type="button" class="primary" onclick={() => onissue?.(template)} disabled={busy}>{t(busy ? 'Issuing…' : 'Issue invoice')}</button>
    {:else}
      <button type="button" onclick={() => dialog.close()}>{t('Return to draft')}</button>
    {/if}
  </footer>
</dialog>

{#if invoice.number}
  <article class={`invoice-paper print-only paper-${template} accent-${layoutSettings.accent}`} aria-hidden="true">{@render paper()}</article>
{/if}

{#snippet paper()}
    <header class="paper-header">
      <div>{#if layoutSettings.logo}<img class="paper-logo" src={layoutSettings.logo} alt="">{/if}
        <h2>{invoice.seller.name}</h2><p class="address">{invoice.seller.address}</p>
        <p>{t(stateLabel(invoice.seller.state, states))}</p>
        {#if invoice.seller.gstin}<p>GSTIN {invoice.seller.gstin}</p>{/if}
      </div>
      <div class="paper-ref"><strong>{t(invoice.seller.gstin ? 'Tax invoice' : 'Invoice')}</strong>
        <p>{invoice.number || t('Draft')}</p><p>{dateLabel(invoice.date, $locale)}</p>
        {#if invoice.currency === 'USD'}<p>{t('US dollar (USD)')}</p>{/if}
        {#if invoice.number}<p>{t(copy)}</p>{/if}
      </div>
    </header>
    <div class="paper-parties">
      <div><strong>{t('Bill to')}</strong><p>{invoice.customer?.name || t('Walk-in customer')}</p>
        {#if invoice.customer?.address}<p class="address">{invoice.customer.address}</p>{/if}
        {#if invoice.customer?.gstin}<p>GSTIN {invoice.customer.gstin}</p>{/if}
        {#if layoutSettings.showContacts && (invoice.customer?.phone || invoice.customer?.email)}
          <p class="address">{[invoice.customer?.phone, invoice.customer?.email].filter(Boolean).join(' · ')}</p>
        {/if}
      </div>
      <div><strong>{t('Place of supply')}</strong><p>{invoice.place} · {t(stateLabel(invoice.place, states))}</p>
        {#if invoice.deliveryAddress}<p class="address">{invoice.deliveryAddress}</p>{/if}
      </div>
    </div>
    <div class="table-wrap">
      <table aria-label={t('Invoice items')}><thead><tr><th scope="col">{t('Item / HSN')}</th><th scope="col" class="numeric">{t('Qty')}</th>{#if !compact}<th scope="col" class="numeric">{t('Rate')}</th><th scope="col" class="numeric">{t('Discount')}</th>{/if}<th scope="col" class="numeric">{t(invoice.seller.gstin ? 'Taxable' : 'Amount')}</th>{#if invoice.seller.gstin}<th scope="col" class="numeric">GST</th>{/if}</tr></thead>
        <tbody>{#each invoice.lines as line}
          <tr><td>{line.name}<small>{[line.hsn, compact ? `${money(line.price, invoice.currency, $locale)}/${line.unit}` : ''].filter(Boolean).join(' · ')}</small>
              {#if compact && line.discount}<small>{t('Discount')} {money(line.discount, invoice.currency, $locale)}</small>{/if}
            </td><td class="numeric">{formatQuantity(line.quantity, $locale)}{#if compact}<small class="paper-unit">{line.unit}</small>{:else} {line.unit}{/if}</td>
            {#if !compact}<td class="numeric">{money(line.price, invoice.currency, $locale)}</td><td class="numeric">{money(line.discount, invoice.currency, $locale)}</td>{/if}
            <td class="numeric">{money(line.taxable, invoice.currency, $locale)}</td>
            {#if invoice.seller.gstin}<td class="numeric">{formatCount(line.rate / 100, $locale)}%<small>{money(line.cgst + line.localGst + line.igst, invoice.currency, $locale)}</small></td>{/if}
          </tr>
        {/each}</tbody>
      </table>
    </div>
    <div class="paper-summary">
      <dl class="paper-totals">
        <div><dt>{t('Subtotal')}</dt><dd>{money(invoice.totals.taxable, invoice.currency, $locale)}</dd></div>
        {#if invoice.seller.gstin}
          {#if invoice.seller.state === invoice.place}
            <div><dt>CGST</dt><dd>{money(invoice.totals.cgst, invoice.currency, $locale)}</dd></div>
            <div><dt>{invoice.localTaxName}</dt><dd>{money(invoice.totals.localGst, invoice.currency, $locale)}</dd></div>
          {:else}
            <div><dt>IGST</dt><dd>{money(invoice.totals.igst, invoice.currency, $locale)}</dd></div>
          {/if}
        {/if}
        <div class="total"><dt>{t('Total')}</dt><dd>{money(invoice.totals.total, invoice.currency, $locale)}</dd></div>
      </dl>
      <footer class="paper-footer">
        <div><p>{t('Payment')} · {t(invoice.payment)}{layoutSettings.showPaymentReference && invoice.reference ? ` · ${invoice.reference}` : ''}</p>
          {#if invoice.seller.gstin}<p>{t('Reverse charge · No')}</p>{/if}
        </div>
        <p class="signature">{t('Authorised signature')}</p>
      </footer>
      {#if layoutSettings.footer}<p class="paper-note">{layoutSettings.footer}</p>{/if}
    </div>
{/snippet}
