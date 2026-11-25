"use strict";

const sampleItems = [
  { id: "drill", name: "Cordless drill kit", hsn: "8467", unit: "Nos", price: 185000, gst: 18 },
  { id: "screws", name: "Stainless steel screw pack", hsn: "7318", unit: "Pack", price: 8000, gst: 18 },
  { id: "tape", name: "Insulation tape", hsn: "3919", unit: "Roll", price: 3500, gst: 18 },
  { id: "pliers", name: "Combination pliers", hsn: "8203", unit: "Nos", price: 32000, gst: 18 }
];

const sampleParties = [
  { id: "roy", name: "Roy Electricals", state: "WB", gstin: "19ABCDE1234F1Z5", address: "Ballygunge, Kolkata, 700019" },
  { id: "sen", name: "Sen Industrial Supply", state: "OD", gstin: "21ABCDE1234F1Z1", address: "Saheed Nagar, Bhubaneswar, 751007" },
  { id: "walk", name: "Walk-in customer", state: "WB", gstin: "", address: "" }
];

const states = {
  JK: "01 · Jammu and Kashmir", HP: "02 · Himachal Pradesh", PB: "03 · Punjab", CH: "04 · Chandigarh",
  UK: "05 · Uttarakhand", HR: "06 · Haryana", DL: "07 · Delhi", RJ: "08 · Rajasthan", UP: "09 · Uttar Pradesh",
  BR: "10 · Bihar", SK: "11 · Sikkim", AR: "12 · Arunachal Pradesh", NL: "13 · Nagaland", MN: "14 · Manipur",
  MZ: "15 · Mizoram", TR: "16 · Tripura", ML: "17 · Meghalaya", AS: "18 · Assam", WB: "19 · West Bengal",
  JH: "20 · Jharkhand", OD: "21 · Odisha", CG: "22 · Chhattisgarh", MP: "23 · Madhya Pradesh", GJ: "24 · Gujarat",
  DH: "26 · Dadra and Nagar Haveli and Daman and Diu", MH: "27 · Maharashtra", KA: "29 · Karnataka",
  GA: "30 · Goa", LD: "31 · Lakshadweep", KL: "32 · Kerala", TN: "33 · Tamil Nadu", PY: "34 · Puducherry",
  AN: "35 · Andaman and Nicobar Islands", TS: "36 · Telangana", AP: "37 · Andhra Pradesh", LA: "38 · Ladakh"
};
const items = structuredClone(sampleItems);
const parties = structuredClone(sampleParties);
const unionTerritories = new Set(["CH", "DH", "LD", "AN", "LA"]);

const demoDate = "2026-11-25";
const workspace = document.querySelector("#workspace");
const nav = document.querySelector("#navigation");
const dialog = document.querySelector("#dialog");
const scenarioSelect = document.querySelector("#scenario");
const toast = document.querySelector("#toast");
const indianMoney = new Intl.NumberFormat("en-IN", { style: "currency", currency: "INR" });

let page = "home";
let scenario = "local";
let nextNumber = 106;
let toastTimeout;
let backupTime = "";
const draftRecords = [];
const postedRecords = [];

function makeLine(item, quantity) {
  return { item: item.id, quantity, price: item.price };
}

function makeDraft() {
  const party = sampleMode ? partyById(scenario === "interstate" ? "sen" : "roy") : partyById("walk");
  return {
    id: "",
    party: party.id,
    pos: party.state,
    date: demoDate,
    payment: business.payment,
    reference: "",
    irnRequired: sampleMode && scenario === "irn",
    lines: sampleMode ? [makeLine(itemById("drill"), 2), makeLine(itemById("screws"), 5)] : []
  };
}

let draft = makeDraft();

const seedRecords = [
  { ...makeDraft(), number: "BH/26-27/0105", date: "2026-11-24", payment: "UPI", status: "Issued" },
  { ...makeDraft(), number: "BH/26-27/0104", date: "2026-11-23", party: "sen", pos: "OD", status: "Issued", lines: [makeLine(items[3], 4)] },
  { ...makeDraft(), number: "BH/26-27/0103", date: "2026-11-21", party: "walk", status: "Issued", payment: "Cash", lines: [makeLine(items[2], 6)] }
];

function resetWorkspaceRecords(useDemo) {
  sampleMode = useDemo;
  scenario = "local";
  nextNumber = useDemo ? 106 : 1;
  backupTime = "";
  draftRecords.length = 0;
  postedRecords.length = 0;
  extraRecords.clear();
  items.splice(0, items.length, ...(useDemo ? structuredClone(sampleItems) : []));
  const customerSeed = useDemo ? structuredClone(sampleParties) : [{ ...sampleParties[2], state: business.state }];
  parties.splice(0, parties.length, ...customerSeed);
  draft = makeDraft();
}

function esc(value) {
  return String(value ?? "").replace(/[&<>"']/g, (char) =>
    ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[char]);
}

function money(paise) { return indianMoney.format(paise / 100); }
function countLabel(count, singular, plural = `${singular}s`) { return `${count} ${count === 1 ? singular : plural}`; }
function itemById(id) { return items.find((item) => item.id === id); }
function partyById(id) { return parties.find((party) => party.id === id); }
function dateText(value) {
  if (!value) return "—";
  return new Intl.DateTimeFormat("en-IN", { dateStyle: "medium", timeZone: "UTC" }).format(new Date(value));
}

function calculate(bill) {
  const intra = bill.pos === business.state;
  const localTaxName = unionTerritories.has(business.state) ? "UTGST" : "SGST";
  const entries = bill.lines.map((line) => {
    const item = itemById(line.item);
    const base = line.quantity * line.price;
    const gst = business.treatment === "regular" ? item.gst : 0;
    const split = Math.round(base * gst / 200);
    const igst = intra ? 0 : Math.round(base * gst / 100);
    return { ...line, name: item.name, gst, base, cgst: intra ? split : 0, sgst: intra ? split : 0, igst };
  });
  const totals = { base: 0, cgst: 0, sgst: 0, igst: 0 };
  for (const entry of entries) {
    for (const key of Object.keys(totals)) totals[key] += entry[key];
  }
  const gross = totals.base + totals.cgst + totals.sgst + totals.igst;
  return { ...totals, entries, intra, gross, localTaxName };
}

function head(title, subtitle, actions = "") {
  return `<div class="page-head"><div><h1>${title}</h1><p class="sub">${subtitle}</p></div>${actions ? `<div class="head-actions">${actions}</div>` : ""}</div>`;
}

function option(value, label, current) {
  return `<option value="${esc(value)}"${value === current ? " selected" : ""}>${esc(label)}</option>`;
}

function billNumber() { return `${business.prefix}/26-27/${String(nextNumber).padStart(4, "0")}`; }

function documentType() {
  if (business.treatment === "composition") return "Bill of supply";
  return business.treatment === "regular" ? "Tax invoice" : "Invoice";
}

function requiresIrn(bill) {
  return bill.irnRequired || (!sampleMode && business.treatment === "regular" && business.irn !== "no" && Boolean(partyById(bill.party).gstin));
}

function render() {
  const section = moduleRoutes.get(page);
  nav.innerHTML = navigationView();
  document.querySelector("#breadcrumb").innerHTML = `Workspace / <strong>${section.title}</strong>`;
  scenarioSelect.value = scenario;
  document.querySelector("#scenario-control").hidden = !(sampleMode && page === "billing");
  document.querySelector(".shop strong").textContent = business.name;
  document.querySelector(".shop small").textContent = `${business.city}, ${stateName(business.state)}`;
  document.querySelector(".shop-mark").textContent = business.name.split(/\s+/).slice(0, 2).map((part) => part[0]).join("");
  const views = { home: homeView, billing, invoices, customers, items: catalog, reports, backup };
  const group = modules.find((entry) => entry.id === page);
  workspace.innerHTML = views[page] ? views[page]() : group ? groupView(group) : genericView(page, section);
  if (page === "billing") updateTotals();
}

function billing() {
  const party = partyById(draft.party);
  const issue = requiresIrn(draft) ? "IRN options" : "Review & issue";
  return head(`New ${documentType().toLowerCase()}`, "Create a sale in your local billing workspace.",
    `<button class="btn" data-action="save">Save draft</button>
     <button class="btn primary" data-action="review">${issue} <kbd>F9</kbd></button>`) +
    `<div class="editor">
      <div class="column">
        <section class="card">
          <div class="card-head"><h2>Invoice details</h2><div class="head-actions"><span class="badge">Draft</span><button class="btn" data-action="add-customer">＋ Customer</button></div></div>
          <div class="card-body fields">
            <label class="field">Customer <select id="party">${parties.map((entry) => option(entry.id, entry.name, draft.party)).join("")}</select></label>
            <label class="field">GSTIN <input value="${esc(party.gstin || "Unregistered customer")}" readonly></label>
            <label class="field">Place of supply <select id="pos">${Object.entries(states).map(([key, label]) => option(key, label, draft.pos)).join("")}</select></label>
            <label class="field">Invoice date <input id="date" type="date" min="2026-04-01" max="2027-03-31" value="${esc(draft.date)}" required></label>
          </div>
        </section>
        <section class="card">
          <div class="card-head"><h2>Line items</h2><span class="badge" id="item-count"></span></div>
          <div class="table-wrap">
            <table class="line-table" aria-label="Invoice line items"><thead><tr><th scope="col">Item / HSN</th><th scope="col" class="num">Qty</th><th scope="col" class="num">Rate ₹</th><th scope="col" class="num">GST</th><th scope="col" class="num">Taxable</th><th scope="col"><span class="hint" aria-hidden="true"></span></th></tr></thead>
              <tbody>${draft.lines.length ? draft.lines.map(lineRow).join("") : '<tr><td colspan="6" class="empty">Add an item to begin.</td></tr>'}</tbody>
            </table>
          </div>
          <div class="add-row"><button class="btn linklike" data-action="picker">＋ Add item <kbd>F8</kbd></button><span class="hint">Rates exclude GST</span></div>
        </section>
      </div>
      <div class="column">
        <section class="card summary">
          <div class="card-head"><h2>Invoice total</h2><span class="badge green">INR</span></div>
          <div class="card-body">
            <dl id="breakdown"></dl>
            <div class="total"><b>Total payable</b><strong class="number" id="grand-total"></strong></div>
            <div class="tax-indicator${requiresIrn(draft) ? " warning" : ""}" id="tax-indicator"></div>
          </div>
        </section>
        <section class="card payment">
          <div class="card-head compact"><h2>Payment</h2></div>
          <div class="card-body">
            <label class="field">Method <select id="payment">${["On credit", "Cash", "UPI", "Bank transfer"].map((name) => option(name, name, draft.payment)).join("")}</select></label>
            <label class="field">Reference <input id="reference" value="${esc(draft.reference)}" placeholder="Optional"></label>
          </div>
        </section>
      </div>
    </div>`;
}

function lineRow(line, index) {
  const item = itemById(line.item);
  return `<tr><td><span class="item-name">${esc(item.name)}</span><small class="item-meta number">${esc(item.hsn)} · ${esc(item.unit)}</small></td>
    <td class="num"><input type="number" class="number" min="1" max="9999" step="1" required value="${line.quantity}" data-quantity="${index}" aria-label="Quantity for ${esc(item.name)}"></td>
    <td class="num"><input type="number" class="number rate" min="0.01" max="99999" step="0.01" required value="${(line.price / 100).toFixed(2)}" data-price="${index}" aria-label="Rate for ${esc(item.name)}"></td>
    <td class="num number">${business.treatment === "regular" ? item.gst + "%" : "—"}</td><td class="num number" data-line-total="${index}"></td>
    <td><button class="icon-btn" data-action="remove" data-index="${index}" aria-label="Remove ${esc(item.name)}">×</button></td></tr>`;
}

function updateTotals() {
  const totals = calculate(draft);
  totals.entries.forEach((entry, index) => {
    workspace.querySelector(`[data-line-total="${index}"]`).textContent = money(entry.base);
  });
  workspace.querySelector("#item-count").textContent = countLabel(totals.entries.length, "item");
  const charges = totals.intra ? [["CGST", totals.cgst], [totals.localTaxName, totals.sgst]] : [["IGST", totals.igst]];
  const breakdown = business.treatment === "regular" ? [["Taxable value", totals.base], ...charges] : [["Subtotal", totals.base]];
  workspace.querySelector("#breakdown").innerHTML = breakdown.map(([label, amount]) =>
    `<div><dt>${label}</dt><dd class="number">${money(amount)}</dd></div>`).join("");
  workspace.querySelector("#grand-total").textContent = money(totals.gross);
  const irn = requiresIrn(draft);
  const heading = irn ? "IRN review before issue" : business.treatment !== "regular" ? "GST is not charged on this document" : totals.intra ? "Intra-state GST" : "Interstate GST";
  const explanation = irn ? "Keep this draft offline until applicability is confirmed and any required IRN and signed QR are available."
    : business.treatment !== "regular" ? treatments[business.treatment] : totals.intra ? `Supplier and place of supply are in ${stateName(business.state)}.` : "The place of supply differs from the supplier’s state.";
  workspace.querySelector("#tax-indicator").innerHTML = `<span aria-hidden="true">${irn ? "!" : "✓"}</span><div><b>${heading}</b>${explanation}</div>`;
}

function table(headers, rows, label) {
  return `<div class="table-wrap"><table class="list" aria-label="${label}"><thead><tr>${headers.map((name) =>
    `<th scope="col">${name}</th>`).join("")}</tr></thead><tbody>${rows}</tbody></table></div>`;
}

function allIssued() { return [...postedRecords, ...(sampleMode ? seedRecords : [])]; }

function invoices() {
  const records = [...draftRecords, ...allIssued()];
  const rows = records.map((record) => {
    const isDraft = record.status !== "Issued";
    const id = isDraft ? record.id : record.number;
    const label = isDraft ? "Unnumbered draft" : record.number;
    return `<tr><td><strong class="number">${esc(label)}</strong></td><td>${esc(partyById(record.party).name)}</td>
      <td>${dateText(record.date)}</td><td><span class="badge ${isDraft ? "amber" : "green"}">${record.status}</span></td>
      <td class="number">${money(calculate(record).gross)}</td>
      <td><button class="btn" data-action="${isDraft ? "edit-draft" : "view-invoice"}" data-id="${esc(id)}">${isDraft ? "Open draft" : "View"}</button></td></tr>`;
  }).join("");
  return head("Invoices", "Review issued documents, reopen drafts and print a copy.", '<button class="btn primary" data-action="new">＋ New invoice</button>') +
    `<section class="card">${rows ? table(["Invoice", "Customer", "Date", "Status", "Total", "Action"], rows, "Invoices") : '<p class="empty">No invoices yet. Create the first invoice to see it here.</p>'}</section>`;
}

function customers() {
  return head("Customers", "Manage contacts and their billing details.", '<button class="btn primary" data-action="add-customer">＋ New customer</button>') +
    `<section class="card">${table(["Customer", "GSTIN", "State", "Action"], parties.map((party) =>
      `<tr><td><b>${esc(party.name)}</b><small class="item-meta">${esc(party.address || "Counter sale")}</small></td>
      <td class="number">${esc(party.gstin || "—")}</td><td>${states[party.state]}</td>
      <td><button class="btn" data-action="bill-party" data-id="${party.id}">Create invoice</button></td></tr>`).join(""), "Customers")}</section>`;
}

function catalog() {
  return head("Items & services", "Set up your catalogue, HSN or SAC, units and tax rates.", '<button class="btn primary" data-action="add-catalog-item">＋ New item</button>') +
    `<section class="card">${items.length ? table(["Item", "HSN / SAC", "Unit", "Rate", "GST"], items.map((item) =>
      `<tr><td><b>${esc(item.name)}</b></td><td class="number">${item.hsn}</td><td>${item.unit}</td>
      <td class="number">${money(item.price)}</td><td class="number">${item.gst}%</td></tr>`).join(""), "Item catalogue") : '<p class="empty">Add the first item or service to use it on an invoice.</p>'}</section>`;
}

function reports() {
  const issued = allIssued();
  const totals = issued.reduce((summary, bill) => {
    const calc = calculate(bill);
    return { taxable: summary.taxable + calc.base, gst: summary.gst + calc.cgst + calc.sgst + calc.igst };
  }, { taxable: 0, gst: 0 });
  const rows = issued.map((bill) => {
    const totals = calculate(bill);
    return `<tr><td class="number">${bill.number}</td><td>${partyById(bill.party).gstin ? "B2B" : "B2C"}</td>
      <td class="number">${money(totals.base)}</td><td class="number">${money(totals.cgst)}</td>
      <td class="number">${money(totals.sgst)}</td><td class="number">${money(totals.igst)}</td></tr>`;
  }).join("");
  return head("GST sales register", "Issued invoices · All dates", '<button class="btn primary" data-action="csv">Export sample CSV</button>') +
    `<div class="notice">Accountant review copy. The sample CSV is for evaluating the desktop workflow.</div>
    <div class="metrics">
      <div class="card metric"><span>Issued invoices</span><strong class="number">${issued.length}</strong></div>
      <div class="card metric"><span>Taxable sales</span><strong class="number">${money(totals.taxable)}</strong></div>
      <div class="card metric"><span>GST collected on sales</span><strong class="number">${money(totals.gst)}</strong></div>
    </div>
    <section class="card"><div class="card-head"><h2>Sales register</h2></div>${rows ? table(["Invoice", "Supply", "Taxable", "CGST", calculate({ pos: business.state, lines: [] }).localTaxName, "IGST"], rows, "GST sales register") : '<p class="empty">Issue an invoice to see its totals here.</p>'}</section>`;
}

function backup() {
  return head("Backup & restore", "Keep a copy of the workspace outside this device.") +
    `<div class="two-cards">
      <section class="card"><h2>Export backup</h2><p class="sub">Download sample records and catalogue as a JSON backup for this prototype.</p>
        <span class="badge ${backupTime ? "green" : "amber"}">${backupTime ? "Export attempted at " + esc(backupTime) : "No sample backup exported this session"}</span>
        <button class="btn primary" data-action="export">Download sample backup</button></section>
      <section class="card"><h2>Check a backup</h2><p class="sub">Choose a sample backup to inspect the file format and see what a restore would include.</p>
        <button class="btn" data-action="choose-backup">Choose sample file</button><input id="backup-file" type="file" accept=".json,application/json" aria-label="Choose sample backup file" hidden></section>
    </div>`;
}

function openDialog(title, body) {
  dialog.innerHTML = `<div class="dialog-head"><h2 id="dialog-title">${title}</h2>
    <button class="icon-btn" data-action="close" aria-label="Close dialog">×</button></div><div class="dialog-body">${body}</div>`;
  if (!dialog.open) dialog.showModal();
}

function showToast(message) {
  clearTimeout(toastTimeout);
  toast.textContent = message;
  toast.hidden = false;
  toastTimeout = setTimeout(() => { toast.hidden = true; }, 4500);
}

function openPage(target) {
  page = target;
  render();
  workspace.focus({ preventScroll: true });
  window.scrollTo(0, 0);
}

function showPicker() {
  if (!items.length) return showMasterForm("item");
  openDialog("Add an item", `<label class="field">Search by name or HSN <input id="item-search" type="search" placeholder="Search catalogue" autofocus></label><div class="pick-list" id="pick-list"></div>`);
  renderPicker("");
  dialog.querySelector("#item-search").focus();
}

function renderPicker(term) {
  const query = term.trim().toLowerCase();
  const found = items.filter((item) => `${item.name} ${item.hsn}`.toLowerCase().includes(query));
  dialog.querySelector("#pick-list").innerHTML = found.length ? found.map((item) =>
    `<button class="pick" data-action="add" data-id="${item.id}"><span><b>${esc(item.name)}</b><small class="item-meta">${item.hsn} · ${item.gst}% GST</small></span><strong class="number">${money(item.price)}</strong></button>`).join("")
    : '<p class="empty">No matching items.</p>';
}

function validateDraft() {
  if (!draft.lines.length) {
    showToast("Add at least one line item.");
    return false;
  }
  if (business.treatment === "composition" && draft.pos !== business.state) {
    showToast("This composition workflow previews local sales only.");
    return false;
  }
  return validateEnteredValues();
}

function validateEnteredValues() {
  const invalidInput = workspace.querySelector("input:invalid");
  if (invalidInput) {
    invalidInput.reportValidity();
    return false;
  }
  return true;
}

function saveDraft() {
  if (!validateEnteredValues()) return;
  draft.id ||= `draft-${Date.now()}`;
  const record = structuredClone(draft);
  record.status = requiresIrn(draft) ? "Needs IRN" : "Draft";
  const existing = draftRecords.findIndex((entry) => entry.id === draft.id);
  if (existing < 0) draftRecords.unshift(record);
  else draftRecords[existing] = record;
  if (dialog.open) dialog.close();
  showToast("Draft saved for this demo session. Find it under Invoices.");
}

function invoiceSheet(bill, status) {
  const party = partyById(bill.party);
  const totals = calculate(bill);
  const taxes = business.treatment !== "regular" ? treatments[business.treatment] : totals.intra ? `CGST ${money(totals.cgst)} · ${totals.localTaxName} ${money(totals.sgst)}` : `IGST ${money(totals.igst)}`;
  return `<article class="preview-sheet${business.printer === "80 mm thermal" ? " receipt-preview" : ""}"><span class="badge amber">${status} · Sample</span><h3>${esc(business.name)}</h3>
    <p class="sub">${documentType()} · ${esc(bill.number || billNumber())} · ${dateText(bill.date)}</p>
    <p class="sub">${esc([business.address, business.city, stateName(business.state)].filter(Boolean).join(", "))}${business.gstin ? ` · GSTIN ${esc(business.gstin)}` : ""}</p>
    <p class="sub"><b>Bill to</b> ${esc(party.name)}${party.gstin ? " · " + esc(party.gstin) : ""}<br>
    ${esc(party.address || "Counter sale")} · Place of supply ${states[bill.pos]}</p>
    ${table(["Item", "Qty", "GST", "Taxable"], totals.entries.map((entry) =>
      `<tr><td>${esc(entry.name)}<small class="item-meta">${itemById(entry.item).hsn}</small></td><td>${entry.quantity}</td><td>${entry.gst}%</td>
      <td class="number">${money(entry.base)}</td></tr>`).join(""), "Sample invoice items")}
    <p class="sub">${taxes} · ${esc(bill.payment)}</p>
    <div class="total"><b>Invoice total</b><strong class="number">${money(totals.gross)}</strong></div>
  </article>`;
}

function review() {
  if (!validateDraft()) return;
  if (requiresIrn(draft)) {
    openDialog("IRN review before issue",
      `<div class="notice">Issuance is paused. Confirm applicability and obtain any required IRN and signed QR from the Invoice Registration Portal when online. This prototype simulates the offline state.</div>
       <div class="dialog-footer"><button class="btn" data-action="close">Back to draft</button><button class="btn primary" data-action="save">Save draft</button></div>`);
    return;
  }
  openDialog("Review sample invoice", invoiceSheet(draft, "Preview") +
    '<div class="dialog-footer"><button class="btn" data-action="close">Back to draft</button><button class="btn primary" data-action="issue">Issue sample invoice</button></div>');
}

function issue() {
  if (requiresIrn(draft) || !validateDraft()) return;
  const record = { ...structuredClone(draft), number: billNumber(), status: "Issued" };
  postedRecords.unshift(record);
  nextNumber += 1;
  const index = draftRecords.findIndex((entry) => entry.id === draft.id);
  if (index >= 0) draftRecords.splice(index, 1);
  draft = makeDraft();
  openPage("invoices");
  viewInvoice(record.number);
  showToast("Sample invoice issued in this demo session.");
}

function viewInvoice(number) {
  const record = allIssued().find((bill) => bill.number === number);
  if (!record) return;
  openDialog("Sample invoice", invoiceSheet(record, "Issued") +
    '<div class="dialog-footer"><button class="btn" data-action="close">Close</button><button class="btn primary" data-action="print">Print sample</button></div>');
}

function download(name, content, type) {
  const url = URL.createObjectURL(new Blob([content], { type }));
  const link = document.createElement("a");
  link.href = url;
  link.download = name;
  link.click();
  setTimeout(() => URL.revokeObjectURL(url), 5000);
}

function exportCsv() {
  const localTax = unionTerritories.has(business.state) ? "UTGST" : "SGST";
  const rows = [["Invoice", "Date", "Customer", "Place of supply", "Taxable INR", "CGST INR", `${localTax} INR`, "IGST INR"]];
  for (const bill of allIssued()) {
    const totals = calculate(bill);
    rows.push([bill.number, bill.date, partyById(bill.party).name, states[bill.pos],
      ...[totals.base, totals.cgst, totals.sgst, totals.igst].map((paise) => (paise / 100).toFixed(2))]);
  }
  const csv = rows.map((row) => row.map((part) => `"${String(part).replace(/^[=+\-@\t\r]/, "'$&").replaceAll('"', '""')}"`).join(",")).join("\r\n");
  download("ascel-bill-sample-sales.csv", csv, "text/csv;charset=utf-8");
  showToast("Sample sales CSV prepared.");
}

function exportBackup() {
  const snapshot = { format: "ascel-bill-prototype", version: 1, business, invoices: allIssued(), drafts: draftRecords, items, parties, moduleDrafts: Object.fromEntries(extraRecords) };
  download("ascel-bill-sample-backup.json", JSON.stringify(snapshot, null, 2), "application/json");
  backupTime = new Intl.DateTimeFormat("en-IN", { hour: "numeric", minute: "2-digit", timeZone: "Asia/Kolkata" }).format(new Date());
  render();
  showToast("Sample backup prepared.");
}

async function checkBackup(file) {
  if (!file) return;
  try {
    if (file.size > 3_000_000) throw new Error("File exceeds the prototype size limit.");
    const data = JSON.parse(await file.text());
    const lists = [data.invoices, data.drafts, data.items, data.parties];
    if (data.format !== "ascel-bill-prototype" || data.version !== 1 || !lists.every(Array.isArray)) throw new Error("Unknown backup");
    const moduleDraftCount = Object.values(data.moduleDrafts || {}).filter(Array.isArray).reduce((count, rows) => count + rows.length, 0);
    openDialog("Backup contents",
      `<p>This sample contains ${countLabel(data.invoices.length, "issued invoice")}, ${countLabel(data.drafts.length, "invoice draft")}, ${countLabel(data.items.length, "item")}, ${countLabel(data.parties.length, "customer")} and ${countLabel(moduleDraftCount, "other module draft")}. The demo only previews the backup.</p>
       <div class="dialog-footer"><button class="btn primary" data-action="close">Done</button></div>`);
  } catch {
    showToast("Choose a valid Ascel Bill prototype backup smaller than 3 MB.");
  }
}

function editDraft(id) {
  const record = draftRecords.find((entry) => entry.id === id);
  if (!record) return;
  draft = structuredClone(record);
  scenario = draft.irnRequired ? "irn" : draft.pos === "WB" ? "local" : "interstate";
  openPage("billing");
}

function selectParty(id) {
  const party = partyById(id);
  draft.party = party.id;
  draft.pos = party.state;
}

function selectScenario(value) {
  scenario = value;
  draft = makeDraft();
  openPage("billing");
  showToast("Sample scenario loaded.");
}

function showMasterForm(kind) {
  const customer = kind === "customer";
  const details = customer ? `<label class="field">State <select name="state">${Object.entries(states).map(([key, label]) => option(key, label, business.state)).join("")}</select></label>
    <label class="field">GSTIN (optional) <input name="gstin" maxlength="15" pattern="[0-9A-Za-z]{15}"></label>
    <label class="field">Address (optional) <input name="address" maxlength="100"></label>` :
    '<label class="field">HSN / SAC <input name="hsn" inputmode="numeric" pattern="[0-9]{4,8}" maxlength="8" required></label><label class="field">Unit <select name="unit"><option>Nos</option><option>Pack</option><option>Roll</option><option>Kg</option><option>Hours</option></select></label><label class="field">Rate in ₹ <input name="price" type="number" min="0.01" max="99999" step="0.01" required></label><label class="field">GST rate <select name="gst"><option value="0">0%</option><option value="5">5%</option><option value="12">12%</option><option value="18" selected>18%</option><option value="28">28%</option><option value="40">40%</option></select></label>';
  openDialog(customer ? "Add a customer" : "Add an item or service", `<form id="master-form" data-kind="${kind}"><div class="fields"><label class="field">${customer ? "Customer" : "Item"} name <input name="name" maxlength="90" required autofocus></label>${details}</div>
    <p class="sub">This record is available for this preview session.</p><div class="dialog-footer"><button type="button" class="btn" data-action="close">Cancel</button><button class="btn primary" type="submit">${customer ? "Save customer" : "Save item"}</button></div></form>`);
}

dialog.addEventListener("submit", (event) => {
  const form = event.target;
  if (form.id !== "master-form") return;
  event.preventDefault();
  const values = new FormData(form);
  const name = String(values.get("name")).trim();
  if (!name) {
    form.elements.name.setCustomValidity("Enter a name.");
    form.elements.name.reportValidity();
    return;
  }
  const id = `record-${Date.now()}`;
  if (form.dataset.kind === "customer") {
    const state = values.get("state");
    const gstin = String(values.get("gstin")).toUpperCase();
    if (gstin && !gstin.startsWith(states[state].slice(0, 2))) {
      form.elements.gstin.setCustomValidity("GSTIN should begin with the selected state code.");
      form.elements.gstin.reportValidity();
      return;
    }
    parties.push({ id, name, state, gstin, address: values.get("address") });
    if (page === "billing") selectParty(id);
  } else {
    const item = { id, name, hsn: values.get("hsn"), unit: values.get("unit"), price: Math.round(Number(values.get("price")) * 100), gst: Number(values.get("gst")) };
    items.push(item);
    if (page === "billing") draft.lines.push(makeLine(item, 1));
  }
  dialog.close();
  render();
  showToast(`${form.dataset.kind === "customer" ? "Customer" : "Item"} saved for this session.`);
});

const actions = {
  save: saveDraft,
  review,
  picker: showPicker,
  issue,
  csv: exportCsv,
  export: exportBackup,
  print: () => window.print(),
  close: () => dialog.close(),
  remove: (button) => { draft.lines.splice(Number(button.dataset.index), 1); render(); },
  add: (button) => { draft.lines.push(makeLine(itemById(button.dataset.id), 1)); dialog.close(); render(); workspace.querySelector(".line-table tbody tr:last-child input").focus(); },
  new: () => { draft = makeDraft(); openPage("billing"); },
  "edit-draft": (button) => editDraft(button.dataset.id),
  "view-invoice": (button) => viewInvoice(button.dataset.id),
  "choose-backup": () => document.querySelector("#backup-file").click(),
  "add-customer": () => showMasterForm("customer"),
  "add-catalog-item": () => showMasterForm("item"),
  "bill-party": (button) => { draft = makeDraft(); selectParty(button.dataset.id); openPage("billing"); }
};

document.addEventListener("click", (event) => {
  const button = event.target.closest("button");
  if (!button) return;
  if (button.dataset.page) openPage(button.dataset.page);
  if (button.dataset.action) (actions[button.dataset.action] || experienceActions[button.dataset.action])?.(button);
});

document.addEventListener("change", (event) => {
  const target = event.target;
  if (target === scenarioSelect) selectScenario(target.value);
  if (target.id === "party") { selectParty(target.value); render(); document.querySelector("#party").focus(); }
  if (target.id === "pos") { draft.pos = target.value; updateTotals(); }
  if (target.id === "date") draft.date = target.value;
  if (target.id === "payment") draft.payment = target.value;
  if (target.id === "backup-file") { checkBackup(target.files[0]); target.value = ""; }
});

document.addEventListener("input", (event) => {
  const target = event.target;
  if (target.form?.id === "master-form") {
    target.form.elements.name.setCustomValidity("");
    target.form.elements.gstin?.setCustomValidity("");
  }
  if (target.id === "item-search") renderPicker(target.value);
  if (target.id === "reference") draft.reference = target.value;
  if (target.dataset.quantity !== undefined && target.value && target.validity.valid) {
    draft.lines[target.dataset.quantity].quantity = Number(target.value);
    updateTotals();
  }
  if (target.dataset.price !== undefined && target.value && target.validity.valid) {
    draft.lines[target.dataset.price].price = Math.round(Number(target.value) * 100);
    updateTotals();
  }
});

document.addEventListener("keydown", (event) => {
  if (page !== "billing" || !onboarding.hidden || dialog.open || event.repeat) return;
  if (event.key === "F8") { event.preventDefault(); showPicker(); }
  if (event.key === "F9") { event.preventDefault(); review(); }
});

if (new URLSearchParams(window.location.search).has("demo")) finishSetup(true);
else openSetup();
