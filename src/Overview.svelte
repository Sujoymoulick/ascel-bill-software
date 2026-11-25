<script lang="ts">
  import { tick } from 'svelte';
  import { dateLabel, money, qty, type OverviewData } from './data';
  import { formatCount, locale, localeTag } from './locale';
  import { overviewLabel } from './overview-copy';
  import './overview.css';

  let {
    data, loading, canViewUsd, onnew, oncustomers, onitems, onstock,
  }: {
    data: OverviewData | null;
    loading: boolean;
    canViewUsd: boolean;
    onnew: () => void;
    oncustomers: (mode?: 'add') => void;
    onitems: (mode?: 'add' | 'attention') => void;
    onstock: (id: string) => void;
  } = $props();

  const periods = [7, 14, 30] as const;
  let period = $state<(typeof periods)[number]>(7);
  let currency = $state<'INR' | 'USD'>('INR');
  let selectedDate = $state('');
  let plot = $state<HTMLDivElement>();
  let showCurrency = $derived(canViewUsd || (data?.sales.USD.invoiceCount ?? 0) > 0);
  let salesCurrency = $derived(showCurrency ? currency : 'INR');
  let days = $derived(data?.sales[salesCurrency].days.slice(-period) ?? []);
  let sales = $derived(days.reduce(
    (sum, day) => ({
      total: sum.total + day.total,
      count: sum.count + day.invoiceCount,
      credit: sum.credit + day.creditSales,
      tax: sum.tax + day.tax,
    }),
    { total: 0, count: 0, credit: 0, tax: 0 },
  ));
  let maxTotal = $derived(Math.max(1, ...days.map((day) => day.total)));
  let selected = $derived(days.find((day) => day.date === selectedDate)
    ?? days.filter((day) => day.invoiceCount).at(-1) ?? days.at(-1));
  let chartLabels = $derived(days.filter((_, index) =>
    index === 0 || index === days.length - 1 || index === Math.floor((days.length - 1) / 2)));
  let stockGroups = $derived(data ? [
    { key: 'goodStock', count: data.inStockCount, colour: 'healthy' },
    { key: 'lowStock', count: data.lowStockCount, colour: 'low' },
    { key: 'noStock', count: data.outOfStockCount, colour: 'critical' },
    { key: 'untracked', count: data.untrackedItemCount, colour: 'untracked' },
  ] as const : []);
  let axisNumberFormat = $derived(new Intl.NumberFormat(localeTag($locale), {
    style: 'currency', currency: salesCurrency, currencyDisplay: 'narrowSymbol',
    notation: 'compact', maximumFractionDigits: 1,
  }));
  let shortDateFormat = $derived(new Intl.DateTimeFormat(localeTag($locale), {
    day: 'numeric', month: 'short', timeZone: 'UTC',
  }));
  let t = $derived((key: Parameters<typeof overviewLabel>[1], values?: Parameters<typeof overviewLabel>[2]) =>
    overviewLabel($locale, key, values));
  let formattedDate = $derived((date: string) => dateLabel(date, $locale));
  let formattedMoney = $derived((amount: number) => money(amount, salesCurrency, $locale));
  let formattedCount = $derived((value: number) => formatCount(value, $locale));

  function shortDate(date: string) {
    return shortDateFormat.format(new Date(date + 'T00:00:00Z'));
  }

  function choosePeriod(value: (typeof periods)[number]) {
    period = value;
    selectedDate = '';
  }

  function chooseCurrency(value: 'INR' | 'USD') {
    currency = value;
    selectedDate = '';
  }

  async function moveDay(event: KeyboardEvent, index: number) {
    let next: number;
    if (event.key === 'ArrowLeft') next = Math.max(0, index - 1);
    else if (event.key === 'ArrowRight') next = Math.min(days.length - 1, index + 1);
    else if (event.key === 'Home') next = 0;
    else if (event.key === 'End') next = days.length - 1;
    else return;
    event.preventDefault();
    selectedDate = days[next].date;
    await tick();
    plot?.querySelectorAll<HTMLButtonElement>('button')[next]?.focus();
  }
</script>

<section class="overview-workspace">
  <header class="overview-heading">
    <h1>{t('overview')}</h1>
    <button type="button" class="primary overview-new" onclick={onnew}>
      <span aria-hidden="true">+</span>{t('newInvoice')}
    </button>
  </header>

  {#if data}
    <section class="overview-sales" aria-labelledby="overview-sales-title" aria-busy={loading}>
      <header class="overview-panel-heading">
        <div>
          <h2 id="overview-sales-title">{t('daily')}</h2>
          {#if days.length}
            <p>{t('range', { start: formattedDate(days[0].date), end: formattedDate(days[days.length - 1].date) })}</p>
          {/if}
        </div>
        <div class="overview-filters">
          {#if showCurrency}
            <div class="overview-segments" role="group" aria-label={t('currency')}>
              <button type="button" aria-pressed={salesCurrency === 'INR'} onclick={() => chooseCurrency('INR')}>INR</button>
              <button type="button" aria-pressed={salesCurrency === 'USD'} onclick={() => chooseCurrency('USD')}>USD</button>
            </div>
          {/if}
          <div class="overview-segments" role="group" aria-label={t('salesRange')}>
            {#each periods as value}
              <button type="button" aria-pressed={period === value} onclick={() => choosePeriod(value)}>
                {t('days', { count: formattedCount(value) })}
              </button>
            {/each}
          </div>
        </div>
      </header>

      <dl class="overview-metrics">
        <div class="overview-lead-metric"><dt>{t('invoiced')}</dt><dd>{formattedMoney(sales.total)}</dd></div>
        <div><dt>{t('issued')}</dt><dd>{formattedCount(sales.count)}</dd></div>
        <div><dt>{t('credit')}</dt><dd>{formattedMoney(sales.credit)}</dd></div>
        <div><dt>{t('tax')}</dt><dd>{formattedMoney(sales.tax)}</dd></div>
      </dl>

      {#if selected && sales.count}
        <figure class="overview-figure">
          <figcaption class="overview-chart-detail">
            <strong>{formattedMoney(selected.total)}</strong>
            <span>{formattedDate(selected.date)} · {t('dayCount', { count: formattedCount(selected.invoiceCount) })}</span>
          </figcaption>
          <div class="overview-chart-wrap">
            <div class="overview-chart-values" aria-hidden="true">
              {#each [maxTotal, maxTotal / 2, 0] as amount}<span>{axisNumberFormat.format(amount / 100)}</span>{/each}
            </div>
            <div class="overview-chart" role="group" aria-label={t('chart')} bind:this={plot}>
              {#each days as day, index (day.date)}
                <button type="button" class="overview-day" class:selected={selected.date === day.date}
                  tabindex={selected.date === day.date ? 0 : -1}
                  aria-label={t('point', { date: formattedDate(day.date), total: formattedMoney(day.total), count: formattedCount(day.invoiceCount) })}
                  aria-pressed={selected.date === day.date}
                  onclick={() => selectedDate = day.date}
                  onfocus={() => selectedDate = day.date}
                  onmouseenter={() => selectedDate = day.date}
                  onkeydown={(event) => moveDay(event, index)}>
                  <span style:--height={(day.total / maxTotal) * 100 + '%'}></span>
                </button>
              {/each}
            </div>
            <div class="overview-chart-axis" aria-hidden="true">
              {#each chartLabels as day (day.date)}<span>{shortDate(day.date)}</span>{/each}
            </div>
          </div>
        </figure>
        <details class="overview-daily">
          <summary>{t('dailyTotals')}</summary>
          <div class="table-wrap">
            <table aria-label={t('dailyTotals')}>
              <thead><tr><th scope="col">{t('date')}</th><th scope="col" class="numeric">{t('issued')}</th><th scope="col" class="numeric">{t('amount')}</th></tr></thead>
              <tbody>
                {#each days as day (day.date)}
                  <tr><th scope="row">{formattedDate(day.date)}</th><td class="numeric">{formattedCount(day.invoiceCount)}</td><td class="numeric">{formattedMoney(day.total)}</td></tr>
                {/each}
              </tbody>
            </table>
          </div>
        </details>
      {:else}
        <p class="overview-empty">{t('noSales', { currency: salesCurrency })}</p>
      {/if}
    </section>

    <div class="overview-business">
      <section class="overview-panel" aria-labelledby="overview-inventory-title">
        <header class="overview-panel-heading">
          <div><h2 id="overview-inventory-title">{t('inventory')}</h2><p>{t('itemCount', { count: formattedCount(data.itemCount) })}</p></div>
          <button type="button" onclick={() => onitems(data.itemCount ? undefined : 'add')}>
            {t(data.itemCount ? 'openItems' : 'addItem')}
          </button>
        </header>

        {#if data.itemCount}
          <div class="overview-meter" aria-hidden="true">
            {#each stockGroups as group (group.key)}
              {#if group.count}<span class={group.colour} style:flex-grow={group.count}></span>{/if}
            {/each}
          </div>
          <dl class="overview-breakdown">
            {#each stockGroups as group (group.key)}
              <div><dt><i class={group.colour} aria-hidden="true"></i>{t(group.key)}</dt><dd>{formattedCount(group.count)}</dd></div>
            {/each}
          </dl>
        {:else}
          <p class="overview-empty">{t('emptyInventory')}</p>
        {/if}
      </section>

      <section class="overview-panel" aria-labelledby="overview-customers-title">
        <header class="overview-panel-heading">
          <div><h2 id="overview-customers-title">{t('customers')}</h2><p>{t('customerCount', { count: formattedCount(data.customerCount) })}</p></div>
          <button type="button" onclick={() => oncustomers(data.customerCount ? undefined : 'add')}>
            {t(data.customerCount ? 'openCustomers' : 'addCustomer')}
          </button>
        </header>
        {#if data.customerCount}
          {@const active = Math.min(data.activeCustomerCount, data.customerCount)}
          <div class="overview-meter" aria-hidden="true">
            {#if active}<span class="healthy" style:flex-grow={active}></span>{/if}
            {#if data.customerCount > active}<span class="untracked" style:flex-grow={data.customerCount - active}></span>{/if}
          </div>
          <dl class="overview-breakdown">
            <div><dt><i class="healthy" aria-hidden="true"></i>{t('activeCustomers', { count: formattedCount(data.periodDays) })}</dt><dd>{formattedCount(data.activeCustomerCount)}</dd></div>
            <div><dt><i class="untracked" aria-hidden="true"></i>{t('otherCustomers')}</dt><dd>{formattedCount(data.customerCount - active)}</dd></div>
          </dl>
        {:else}
          <p class="overview-empty">{t('noCustomers')}</p>
        {/if}
      </section>
    </div>

    <section class="overview-panel overview-attention" aria-labelledby="overview-attention-title">
      <header class="overview-panel-heading">
        <div>
          <h2 id="overview-attention-title">{t('stockReview')}</h2>
          <p>{t('threshold', { count: qty(data.reorderThreshold, $locale) })}</p>
        </div>
        {#if data.stockAlerts.length}
          <span class="overview-alert-count">{t('stockAlerts', { count: formattedCount(data.lowStockCount + data.outOfStockCount) })}</span>
        {/if}
      </header>
      {#if data.stockAlerts.length}
        <ul class="overview-stock-list">
          {#each data.stockAlerts as item (item.id)}
            <li>
              <div class="overview-stock-copy">
                <strong>{item.name}</strong>
                <span class:critical={item.stock === 0}>{item.stock === 0 ? t('noStock') : t('remaining', { count: qty(item.stock, $locale), unit: item.unit })}</span>
              </div>
              <button type="button" aria-label={t('updateItem', { item: item.name })} onclick={() => onstock(item.id)}>{t('updateStock')}</button>
            </li>
          {/each}
        </ul>
        {#if data.lowStockCount + data.outOfStockCount > data.stockAlerts.length}
          <button type="button" class="text-button overview-more" onclick={() => onitems('attention')}>
            {t('stockRemainder', { count: formattedCount(data.lowStockCount + data.outOfStockCount - data.stockAlerts.length) })}
          </button>
        {/if}
      {:else}
        <p class="overview-empty">{t(data.trackedItemCount ? 'readyStock' : 'noTrackedStock')}</p>
      {/if}
    </section>
  {:else}
    <p role="status" class="overview-panel overview-empty">{t(loading ? 'loading' : 'unavailable')}</p>
  {/if}
</section>
