<script lang="ts">
  import { api, errorText, normalize } from './data';
  import type { Business, StateOption } from './data';
  import { languageOptions, locale, setLocale, translator } from './locale';
  import logo from '../src-tauri/icons/brand.png';

  let { states, onready, onrestore }: {
    states: StateOption[];
    onready: (business: Business) => void;
    onrestore: () => void;
  } = $props();

  let step = $state(1);
  let registered = $state(true);
  let name = $state('');
  let address = $state('');
  let stateCode = $state('');
  let gstin = $state('');
  let prefix = $state('INV');
  let irn = $state('');
  let busy = $state(false);
  let error = $state('');
  let t = $derived(translator($locale));

  async function next(event: SubmitEvent) {
    event.preventDefault();
    error = '';
    if (step === 1) {
      if (!name.trim() || !address.trim()) {
        error = 'Enter the business name and address.';
        return;
      }
      step = 2;
      return;
    }
    if (busy) return;
    busy = true;
    try {
      const business = await api.saveBusiness({
        name: name.trim(), address: address.trim(), state: stateCode, prefix: normalize(prefix), gstin: registered ? normalize(gstin) : '',
        irnRequired: registered && irn !== 'no',
      });
      onready(business);
    } catch (cause) {
      error = errorText(cause);
    } finally {
      busy = false;
    }
  }
</script>

<div class="setup-screen">
  <section class="setup-card" aria-labelledby="setup-title">
    <header class="setup-brand"><img src={logo} alt="" width="34" height="34"><strong>Ascel Bill</strong></header>
    <div class="setup-steps" aria-label={t('Setup progress')}>
      <span aria-current={step === 1 ? 'step' : undefined}>{t('1. Business')}</span>
      <span aria-current={step === 2 ? 'step' : undefined}>{t('2. GST and numbering')}</span>
    </div>
    <form onsubmit={next}>
      <div class="setup-body">
        <h1 id="setup-title">{t(step === 1 ? 'Set up your business' : 'Set up invoices')}</h1>
        {#if step === 1}
          <label>{t('Language')}
            <select value={$locale} onchange={(event) => setLocale(event.currentTarget.value)}>
              {#each languageOptions as language}<option value={language.value} lang={language.value}>{language.label}</option>{/each}
            </select>
          </label>
          <label>{t('Business name')} <input bind:value={name} maxlength="100" required></label>
          <label>{t('Address for invoices')} <textarea bind:value={address} maxlength="350" rows="3" required></textarea></label>
          <label>{t('State or union territory')}
            <select bind:value={stateCode} required>
              <option value="">{t('Select state')}</option>
              {#each states as [code, title]}<option value={code}>{t(title)}</option>{/each}
            </select>
          </label>
        {:else}
          <label>{t('GST registration')}
            <select bind:value={registered}>
              <option value={true}>{t('Regular GST registration')}</option>
              <option value={false}>{t('Unregistered')}</option>
            </select>
          </label>
          {#if registered}
            <label>GSTIN <input bind:value={gstin} maxlength="15" pattern={'[0-9A-Za-z]{15}'} required autocapitalize="characters"></label>
            <label>{t('Does your business need e-invoicing for B2B sales?')}
              <select bind:value={irn} required>
                <option value="">{t('Choose an option')}</option>
                <option value="no">{t('No, confirmed')}</option>
                <option value="yes">{t('Yes')}</option>
                <option value="unknown">{t('I need to confirm')}</option>
              </select>
            </label>
            {#if irn && irn !== 'no'}<p class="help">{t('You can save B2B drafts. Issuing those invoices requires your approved e-invoicing workflow.')}</p>{/if}
          {/if}
          <label>{t('Invoice prefix')} <input bind:value={prefix} pattern={'[0-9A-Za-z]{1,5}'} maxlength="5" required></label>
        {/if}
        {#if error}<p class="error" role="alert">{t(error)}</p>{/if}
      </div>
      <footer class="setup-footer">
        {#if step === 1}
          <button type="button" class="text-button" onclick={onrestore}>{t('Restore a backup')}</button>
        {:else}
          <button type="button" onclick={() => { step = 1; error = ''; }} disabled={busy}>{t('Back')}</button>
        {/if}
        <button class="primary" type="submit" disabled={busy}>{t(step === 1 ? 'Continue' : busy ? 'Saving…' : 'Open workspace')}</button>
      </footer>
    </form>
  </section>
</div>
