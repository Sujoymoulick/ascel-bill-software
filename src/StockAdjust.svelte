<script lang="ts">
  import { onMount } from 'svelte';
  import { api, errorText, qty } from './data';
  import type { Item, StockMode } from './data';
  import { locale, translator } from './locale';

  let { item, onclose, onsaved }: { item: Item; onclose: () => void; onsaved: (item: Item) => void } = $props();
  let t = $derived(translator($locale));
  let modal: HTMLDialogElement;
  let mode = $state<StockMode>('receive');
  let amount = $state<number | undefined>(undefined);
  let note = $state('');
  let busy = $state(false);
  let error = $state('');
  let next = $derived(amount === undefined ? null : mode === 'receive' ? (item.stock || 0) + Math.round(amount * 1000) : Math.round(amount * 1000));

  onMount(() => modal.showModal());

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (busy || amount === undefined) return;
    busy = true;
    error = '';
    try {
      const saved = await api.updateStock(item.id, mode, Math.round(amount * 1000), note.trim());
      onsaved(saved);
      modal.close();
    } catch (cause) {
      error = errorText(cause);
    } finally {
      busy = false;
    }
  }
</script>

<dialog class="stock-dialog" bind:this={modal} aria-labelledby="stock-title" onclose={onclose}
  oncancel={(event) => { if (busy) event.preventDefault(); }}>
  <header class="dialog-head">
    <div><h2 id="stock-title">{t('Update stock')}</h2><p class="help">{item.name}</p></div>
    <button type="button" onclick={() => modal.close()} disabled={busy}>{t('Close')}</button>
  </header>
  <form onsubmit={submit}>
    <div class="stock-form">
      <p class="stock-current"><span>{t('Current stock')}</span><strong>{item.stock === null ? t('Not tracked') : `${qty(item.stock, $locale)} ${t(item.unit)}`}</strong></p>
      <label>{t('Stock action')}
        <select bind:value={mode} onchange={() => { amount = undefined; error = ''; }}>
          <option value="receive">{t('Receive stock')}</option>
          <option value="count">{t('Set counted quantity')}</option>
        </select>
      </label>
      <label>{mode === 'receive' ? t('Quantity received') : t('Quantity counted')}
        <input type="number" min={mode === 'receive' ? '0.001' : '0'} max="1000000" step="0.001" bind:value={amount} required>
      </label>
      <label>{t('Note (optional)')} <input maxlength="180" bind:value={note}></label>
      {#if next !== null}<p class="stock-current"><span>{t('Stock after update')}</span><strong>{qty(next, $locale)} {t(item.unit)}</strong></p>{/if}
      {#if error}<p class="error" role="alert">{t(error)}</p>{/if}
    </div>
    <footer class="dialog-actions">
      <button type="button" onclick={() => modal.close()} disabled={busy}>{t('Cancel')}</button>
      <button type="submit" class="primary" disabled={busy}>{busy ? t('Saving…') : t('Save stock update')}</button>
    </footer>
  </form>
</dialog>
