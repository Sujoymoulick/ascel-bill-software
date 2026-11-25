<script lang="ts">
  import { api, errorText } from './data';
  import { formatCount, locale, translator } from './locale';
  import type { ImportSummary } from './data';

  let { onDone, onClose }: { onDone: () => void; onClose: () => void } = $props();
  let content = '';
  let filename = $state('');
  let summary = $state<ImportSummary | null>(null);
  let busy = $state(false);
  let applied = $state(false);
  let error = $state('');
  let t = $derived(translator($locale));

  async function choose(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    input.value = '';
    filename = file.name;
    content = '';
    summary = null;
    applied = false;
    error = '';
    busy = true;
    try {
      if (file.size > 1_000_000) throw new Error('Choose a UTF-8 CSV file up to 1 MB.');
      try { content = new TextDecoder('utf-8', { fatal: true }).decode(await file.arrayBuffer()); }
      catch { throw new Error('Save the file as UTF-8 CSV and choose it again.'); }
      summary = await api.importCustomers(content);
    } catch (cause) {
      error = errorText(cause);
    } finally {
      busy = false;
    }
  }

  async function apply() {
    if (busy || !summary?.ready || applied) return;
    busy = true;
    error = '';
    try {
      summary = await api.importCustomers(content, true);
      applied = true;
      onDone();
    } catch (cause) {
      error = errorText(cause);
    } finally {
      busy = false;
    }
  }
</script>

<section class="panel entry-form">
  <h2>{t('Import customers')}</h2>
  <p class="help">{t('Use a UTF-8 CSV with name and phone or email. Optional columns are gstin, state and address.')}</p>
  <label>{t('Customer CSV')} <input type="file" accept=".csv,text/csv" disabled={busy} onchange={choose}></label>
  {#if error}<p class="error" role="alert">{t(error)}</p>{/if}
  {#if summary}
    <div aria-live="polite">
      <strong>{filename}</strong>
      <p class="help">{t(applied ? '{count} imported' : '{count} ready to import', { count: formatCount(applied ? summary.imported : summary.ready, $locale) })} · {t('{count} duplicates', { count: formatCount(summary.duplicates, $locale) })} · {t('{count} invalid rows', { count: formatCount(summary.invalid, $locale) })}</p>
    </div>
    {#if summary.issues.length}
      <ul class="import-issues">
        {#each summary.issues as issue}<li>{t('Row {count}', { count: formatCount(issue.line, $locale) })} · {t(issue.message)}</li>{/each}
      </ul>
    {/if}
  {/if}
  <div class="actions">
    <button type="button" onclick={onClose} disabled={busy}>{t(applied ? 'Done' : 'Close')}</button>
    {#if summary && !applied}
      <button class="primary" type="button" onclick={apply} disabled={busy || !summary.ready}>{t(summary.ready === 1 ? 'Import {count} customer' : 'Import {count} customers', { count: formatCount(summary.ready, $locale) })}</button>
    {/if}
  </div>
</section>
