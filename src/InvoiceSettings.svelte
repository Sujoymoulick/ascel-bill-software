<script lang="ts">
  import { untrack } from 'svelte';
  import { locale, translator } from './locale';
  import { api, defaultPrintSettings, errorText, invoiceTemplates } from './data';
  import type { Business, PrintSettings } from './data';

  let { settings, onSaved }: { settings: PrintSettings; onSaved: (business: Business) => void } = $props();
  let form = $state<PrintSettings>(untrack(() => ({ ...settings })));
  let saving = $state(false);
  let loadingLogo = $state(false);
  let error = $state('');
  let status = $state('');
  let t = $derived(translator($locale));

  async function pickLogo(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = '';
    if (!file) return;
    error = '';
    status = '';
    loadingLogo = true;
    try {
      if (file.size > 256 * 1024 || !['image/png', 'image/jpeg'].includes(file.type)) {
        throw new Error('Choose a PNG or JPEG logo up to 256 KB.');
      }
      const reader = new FileReader();
      form.logo = await new Promise<string>((resolve, reject) => {
        reader.onload = () => resolve(String(reader.result));
        reader.onerror = () => reject(new Error('The logo could not be read.'));
        reader.readAsDataURL(file);
      });
    } catch (cause) {
      error = errorText(cause);
    } finally {
      loadingLogo = false;
    }
  }

  async function save(event: SubmitEvent) {
    event.preventDefault();
    if (saving || loadingLogo) return;
    saving = true;
    error = '';
    status = '';
    try {
      const business = await api.savePrintSettings({ ...$state.snapshot(form), footer: form.footer.trim() });
      form = { ...(business.printSettings || defaultPrintSettings) };
      onSaved(business);
      status = 'Invoice settings saved.';
    } catch (cause) {
      error = errorText(cause);
    } finally {
      saving = false;
    }
  }
</script>

<form class="panel entry-form" onsubmit={save}>
  <h2>{t('Invoice and print')}</h2>
  <div class="fields">
    <label>{t('Default template')}
      <select bind:value={form.template}>
        {#each Object.entries(invoiceTemplates) as [key, template]}
          <option value={key}>{t(template.label)} · {template.size}</option>
        {/each}
      </select>
    </label>
    <label>{t('Accent colour')}
      <select bind:value={form.accent}>
        <option value="green">{t('Green')}</option><option value="navy">{t('Navy')}</option><option value="black">{t('Black')}</option>
      </select>
    </label>
  </div>
  <div class="logo-setting">
    {#if form.logo}<img src={form.logo} alt={t('Invoice logo preview')}>{/if}
    <label>{t('Business logo')}
      <input type="file" accept="image/png,image/jpeg" disabled={saving || loadingLogo} onchange={pickLogo}>
    </label>
    {#if form.logo}<button type="button" onclick={() => form.logo = ''} disabled={saving || loadingLogo}>{t('Remove logo')}</button>{/if}
  </div>
  <label>{t('Footer note')}
    <textarea bind:value={form.footer} rows="2" maxlength="280" placeholder={t('For example, return terms or a thank-you note')}></textarea>
  </label>
  <div class="checklist">
    <label class="check-field"><input type="checkbox" bind:checked={form.showContacts}> {t("Show the customer's phone and email")}</label>
    <label class="check-field"><input type="checkbox" bind:checked={form.showPaymentReference}> {t('Show the payment reference')}</label>
  </div>
  {#if error}<p class="error" role="alert">{t(error)}</p>{/if}
  {#if status}<p class="success" role="status">{t(status)}</p>{/if}
  <div class="actions">
    <button type="button" onclick={() => { form = { ...defaultPrintSettings }; status = ''; error = ''; }} disabled={saving || loadingLogo}>{t('Reset defaults')}</button>
    <button type="submit" class="primary" disabled={saving || loadingLogo}>{t(saving ? 'Saving…' : 'Save invoice settings')}</button>
  </div>
</form>
