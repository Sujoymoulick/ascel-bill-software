"use strict";

const demoBusiness = {
  name: "Bengal Hardware", state: "WB", city: "Kolkata", address: "", type: "Goods",
  treatment: "regular", gstin: "19DEMOX0000X1ZZ", irn: "no", prefix: "BH",
  printer: "A4 invoice", payment: "On credit", financialYear: "2026-04-01",
  openingDate: "2026-11-25", openingCash: "", data: "sample"
};
let business = { ...demoBusiness };
let sampleMode = true;
const modules = [
  { id: "sales", title: "Sales", icon: "▤", description: "From a customer enquiry through billing, dispatch and collection.", entries: [
    ["customers", "Customers", "Manage customer details, GST treatment, addresses and contacts."],
    ["quotes", "Quotes", "Prepare a quotation and convert approved lines to an invoice."],
    ["sales-orders", "Sales orders", "Confirm orders and track fulfilment before invoicing."],
    ["invoices", "Invoices", "Create B2B and B2C invoices and review drafts, print copies and payment status."],
    ["recurring-invoices", "Recurring invoices", "Preview schedules and the drafts they would create on a billing date."],
    ["retainers", "Retainer invoices", "Track advances and how they would be applied to final invoices."],
    ["delivery-challans", "Delivery challans", "Prepare dispatch documents and track invoice conversion."],
    ["payments-received", "Payments received", "Review receipts for cash, UPI, bank and other offline payment methods."],
    ["credit-notes", "Credit notes", "Draft an adjustment against an issued invoice and trace the original document."]
  ]},
  { id: "purchases", title: "Purchases", icon: "▣", description: "Track what you buy, what suppliers bill and what you still owe.", entries: [
    ["vendors", "Vendors", "Maintain vendor contacts, GSTIN, addresses and payment terms."],
    ["purchase-orders", "Purchase orders", "Send purchase requests and track order-to-bill conversion."],
    ["receipts", "Purchase receipts", "Check received quantities against orders and bills."],
    ["bills", "Supplier bills", "Record vendor invoices, due dates and input GST details."],
    ["expenses", "Expenses", "Record spending by category and review receipt attachments."],
    ["recurring-expenses", "Recurring expenses", "Set a schedule for operational costs such as rent."],
    ["recurring-bills", "Recurring bills", "Review draft schedules for supplier bills that repeat."],
    ["payments-made", "Payments made", "Track how and when vendor balances were settled."],
    ["vendor-credits", "Vendor credits", "Link supplier credit notes to bills or returned goods."],
    ["bill-of-entry", "Bills of entry", "Review the workflow for tracking import declarations and associated tax."]
  ]},
  { id: "stock", title: "Items & stock", icon: "◇", description: "Manage what you sell, invoice prices and available inventory.", entries: [
    ["items", "Items & services", "Set the catalogue, HSN or SAC, units, selling prices and GST rates."],
    ["price-lists", "Price lists", "Review retail, wholesale and customer-specific pricing."],
    ["adjustments", "Inventory adjustments", "Track quantity changes with a reason and date."],
    ["warehouses", "Locations & transfers", "Design preview of stock locations and inter-location transfers."],
    ["inventory-valuation", "Inventory valuation", "Review stock value, quantity and costing assumptions."]
  ]},
  { id: "banking", title: "Banking", icon: "▥", description: "Review cash, bank activity and the reconciliation workflow.", entries: [
    ["bank-accounts", "Bank & cash accounts", "Track payment accounts and opening balances."],
    ["bank-statements", "Statement import", "Review imported transactions with unmatched and categorized states."],
    ["reconciliation", "Reconciliation", "Compare statement balances with recorded receipts and payments."],
    ["bank-feeds", "Bank feeds", "See where a connected bank would supply transactions.", "online"]
  ]},
  { id: "tax", title: "GST & compliance", icon: "◫", description: "Prepare GST records locally and see where government portal access comes in.", entries: [
    ["reports", "GST sales register", "Break down issued invoices by supply type and tax components."],
    ["gstr1", "GSTR-1 & GSTR-1A", "Review outward supplies, HSN summaries and amendments before filing."],
    ["gstr3b", "GSTR-3B", "Preview a period summary for liability, input tax credit and payments."],
    ["gstr2b", "GSTR-2B reconciliation", "Compare government purchase data with supplier bills.", "online"],
    ["ims", "Invoice Management System", "Preview review states for GSTN inward transactions.", "online"],
    ["einvoicing", "E-invoicing / IRN", "Review drafts awaiting IRN and signed QR before issue.", "online"],
    ["eway", "E-way bills", "Review dispatch status and where to record a government e-way bill reference.", "online"],
    ["composition", "Composition & CMP-08", "Preview bills of supply and the composition statement workflow."],
    ["tax-settings", "Tax rates & preferences", "Review GST treatment, tax rates, place of supply and reverse charge options."]
  ]},
  { id: "accountant", title: "Accountant", icon: "▧", description: "Give the accountant access to balances, ledgers and review controls.", entries: [
    ["coa", "Chart of accounts", "View the proposed asset, liability, income and expense categories."],
    ["balances", "Opening balances", "Review migration date and opening balances for ledger accounts."],
    ["journals", "Manual journals", "Preview balanced debit and credit entries and supporting references."],
    ["recurring-journals", "Recurring journals", "Review recurring accrual or allocation schedules."],
    ["locking", "Period locking", "Preview how to close a reconciled period to later edits."],
    ["audit", "Activity & audit trail", "Trace who created, reviewed or changed a record in the design."],
    ["approvals", "Transaction approvals", "See where makers and reviewers would approve transactions."]
  ]},
  { id: "analysis", title: "Reports", icon: "▦", description: "Find operational, GST and financial reports by purpose.", entries: [
    ["sales-report", "Sales by customer & item", "Review sales volume by customer, item or period."],
    ["ar-aging", "Receivables ageing", "Review outstanding customer balances by due-date bucket."],
    ["ap-aging", "Payables ageing", "See supplier balances due now and overdue."],
    ["pnl", "Profit & loss", "Preview revenue, cost of sales and operating expenses."],
    ["balance-sheet", "Balance sheet", "Preview assets, liabilities and equity as of a date."],
    ["cashflow", "Cash flow statement", "Review cash received and spent across business activities."],
    ["trial-balance", "Trial balance & general ledger", "Preview ledger balances and drill into transaction references."]
  ]},
  { id: "operations", title: "Projects & tools", icon: "⊞", description: "Explore optional service billing, document and connected tools.", entries: [
    ["projects", "Projects & timesheets", "Preview billable time and expenses by project and customer."],
    ["documents", "Documents & receipts", "Review how attachments could sit beside a bill or expense."],
    ["customer-portal", "Customer portal", "Preview client access to quotes, invoices and self-service payments.", "online"],
    ["online-payments", "Online payments", "Review UPI and payment-gateway setup before enabling live collection.", "online"],
    ["automation", "Automation & reminders", "Design the cadence for payment reminders and recurring jobs.", "online"]
  ]},
  { id: "settings", title: "Settings", icon: "⚙", description: "Manage the business profile and how the local workspace is used.", entries: [
    ["profile", "Business profile", "View the organisation, state, currency and registration selected during setup."],
    ["numbering", "Invoice numbering & templates", "Review document prefixes, paper layouts and invoice defaults."],
    ["users", "Users & roles", "Preview cashier, owner and accountant access."],
    ["preferences", "Preferences & custom fields", "Preview module switches, tags and extra fields on records."],
    ["data-import", "Import & opening data", "Review common CSV imports and inspect a sample file."],
    ["backup", "Backup & restore", "Download a sample file and inspect what a restore would include."],
    ["devices", "Devices & sync", "Review how local billing and an optional connected device would fit together.", "online"]
  ]}
];

const specialRoutes = new Map([
  ["home", { title: "Overview", group: "" }],
  ["billing", { title: "New invoice", group: "sales" }]
]);
const moduleRoutes = new Map(specialRoutes);
for (const group of modules) {
  moduleRoutes.set(group.id, { title: group.title, group: group.id, description: group.description });
  for (const [id, title, description, mode] of group.entries) {
    moduleRoutes.set(id, { title, description, mode, group: group.id });
  }
}

const setupSteps = [
  ["Getting started", "Choose how to begin"],
  ["Business profile", "Name, location and activity"],
  ["GST & billing", "Tax, numbering and print"],
  ["Opening data", "Import or start fresh"],
  ["Review & launch", "Confirm your workspace"]
];
const onboarding = document.querySelector("#onboarding");
let wizardStep = 0;
let chosenStart = "new";
let setup = { ...demoBusiness, name: "", city: "", gstin: "", prefix: "INV", irn: "unknown", data: "blank" };
let importPreview = null;
let importError = "";
let importTarget = "items";
let importRequest = 0;

const treatments = { regular: "Regular GST", composition: "Composition scheme", unregistered: "Not GST-registered" };
const irnLabels = { no: "IRN not required for this demo", yes: "IRN required for eligible B2B", unknown: "Check applicability before B2B issue" };

function openSetup() {
  document.body.classList.add("onboarding-active");
  document.querySelector("#workspace-sidebar").hidden = true;
  document.querySelector("#workspace-shell").hidden = true;
  const skip = document.querySelector("#skip-link");
  skip.href = "#onboarding";
  skip.textContent = "Skip to setup";
  onboarding.hidden = false;
  renderSetup();
  onboarding.querySelector("h1").focus();
}

function setupHeader(title, description) {
  return `<header class="setup-title"><h1 tabindex="-1">${title}</h1><p>${description}</p></header>`;
}

function setupChoice(name, value, current, title, description, icon) {
  return `<label class="setup-choice"><input type="radio" name="${name}" value="${value}"${value === current ? " checked" : ""}>
    <span class="choice-icon" aria-hidden="true">${icon}</span><span class="choice-copy"><strong>${title}</strong><small>${description}</small></span></label>`;
}

function setupInput(label, name, options = "") {
  return `<label class="field">${label}<input name="${name}" value="${esc(setup[name])}" ${options}></label>`;
}

function setupSelect(label, name, options) {
  return `<label class="field">${label}<select name="${name}">${Object.entries(options).map(([value, title]) => option(value, title, setup[name])).join("")}</select></label>`;
}

function importOptions() {
  return `<section class="import-box"><div class="import-top"><label class="field">Import type <select id="import-target">${Object.entries(importTemplates).map(([key, value]) => option(key, value.title, importTarget)).join("")}</select></label>
    <button type="button" class="btn" data-action="import-template">Download sample CSV</button></div>
    <label class="field">Choose a sample file <input type="file" id="setup-import" accept=".csv,.json,text/csv,application/json" aria-describedby="import-feedback"></label>
    <p class="hint">Preview a CSV or an Ascel Bill JSON sample backup. Use fictional data.</p>
    <div id="import-feedback">${importFeedback()}</div></section>`;
}

function setupContent() {
  if (wizardStep === 0) return setupHeader("Set up your billing workspace", "Choose how you want to start with Ascel Bill.") +
    `<div class="choices">
      ${setupChoice("start", "new", chosenStart, "Create a business", "Enter your business details and start billing from a clean workspace.", "＋")}
      ${setupChoice("start", "import", chosenStart, "Move existing records", "Set your migration date and preview a CSV or sample backup.", "⇧")}
      ${setupChoice("start", "sample", chosenStart, "Explore with sample data", "Open a hardware shop with customers, invoices and GST examples.", "▤")}
    </div>`;
  if (wizardStep === 1) return setupHeader("Tell us about your business", "These details appear on your invoice and in your local workspace.") +
    `<div class="fields setup-fields">
      ${setupInput("Business name", "name", 'required maxlength="70" placeholder="e.g. Bengal Hardware"')}
      ${setupSelect("Business activity", "type", { Goods: "Sell goods", Services: "Offer services", Both: "Goods and services" })}
      ${setupSelect("State or union territory", "state", states)}
      ${setupInput("City", "city", 'required maxlength="50" placeholder="e.g. Kolkata"')}
      <div class="span-two">${setupInput("Street address (optional)", "address", 'maxlength="100" placeholder="Shop number, street, postcode"')}</div>
      <label class="field">Country <input value="India" readonly></label>
      <label class="field">Currency <input value="INR · Indian Rupee" readonly></label>
    </div><button type="button" class="btn linklike setup-helper" data-action="sample-profile">Fill with fictional details</button>`;
  if (wizardStep === 2) return setupHeader("Set your GST and billing defaults", "Choose your registration type, then set how invoices are numbered and presented.") +
    `<div class="fields setup-fields">
      ${setupSelect("GST registration", "treatment", treatments)}
      <div id="gstin-wrap"${setup.treatment === "unregistered" ? " hidden" : ""}>${setupInput("GSTIN", "gstin", `${setup.treatment === "unregistered" ? "disabled" : 'required pattern="[0-9A-Za-z]{15}"'} maxlength="15" placeholder="15 characters" aria-describedby="gstin-help"`)}<small class="hint" id="gstin-help">Format and state code are checked in this demo.</small></div>
      <div class="span-two" id="irn-wrap"${setup.treatment === "regular" ? "" : " hidden"}>${setupSelect("E-invoicing applicability", "irn", irnLabels)}</div>
      ${setupInput("Document prefix", "prefix", 'required maxlength="5" pattern="[A-Za-z0-9]{1,5}" placeholder="INV"')}
      ${setupSelect("Preferred print layout", "printer", { "A4 invoice": "A4 invoice", "80 mm thermal": "80 mm thermal (layout preview)" })}
      ${setupSelect("Default payment method", "payment", { "On credit": "On credit", Cash: "Cash", UPI: "UPI", "Bank transfer": "Bank transfer" })}
      <label class="field">Financial year <input value="1 Apr 2026 – 31 Mar 2027" readonly></label>
    </div>`;
  if (wizardStep === 3) return setupHeader("Bring in your opening data", "Set a start date. You can add accounts, products and contacts after setup.") +
    `<div class="fields setup-fields">
      ${setupInput("Opening balance date", "openingDate", 'type="date" required min="2026-04-01" max="2027-03-31"')}
      ${setupInput("Opening cash in ₹ (optional)", "openingCash", 'type="number" min="0" max="1000000000" step="0.01" placeholder="0.00"')}
    </div><div class="choices compact-choices">
      ${setupChoice("data", "blank", setup.data, "Start with blank records", "Add customers, items and opening balances as you go.", "＋")}
      ${setupChoice("data", "import", setup.data, "Preview an import", "Inspect columns and rows from a sample export before migration.", "⇧")}
    </div><div id="setup-import-wrap"${setup.data === "import" ? "" : " hidden"}>${importOptions()}</div>`;
  return setupHeader("Review your workspace", "Confirm your first-launch settings before opening the workspace.") +
    `<div class="review-block">${[
      ["Business", setup.name, 1], ["Location", `${setup.city}, ${stateName(setup.state)}`, 1],
      ["GST registration", treatments[setup.treatment], 2],
      ["Document format", `${setup.prefix.toUpperCase()}/26-27/0001 · ${setup.printer}`, 2],
      ["Opening date", dateText(setup.openingDate), 3],
      ["Records", setup.data === "import" && importPreview ? `${importPreview.file} · ${countLabel(importPreview.rows, "row")} to review` : "Start with empty registers", 3]
    ].map(([label, value, step]) => `<div><span>${label}</span><strong>${esc(value)}</strong><button type="button" class="btn linklike" data-action="setup-edit" data-id="${step}" aria-label="Edit ${label.toLowerCase()}">Edit</button></div>`).join("")}</div>
    ${setup.data === "import" && importPreview ? '<p class="sub">Your file was inspected. The prototype opens a blank workspace so you can continue the setup review safely.</p>' : ""}`;
}

function renderSetup() {
  const first = wizardStep === 0;
  const last = wizardStep === setupSteps.length - 1;
  onboarding.innerHTML = `<div class="onboard-shell">
    <aside class="setup-rail">
      <div class="brand"><span class="brand-mark" aria-hidden="true">₹</span><span>Ascel Bill</span></div>
      <div><p class="rail-label">FIRST LAUNCH</p><h2>Workspace setup</h2></div>
      <ol>${setupSteps.map(([title, description], index) =>
        `<li data-step-state="${index === wizardStep ? "current" : index < wizardStep ? "complete" : "pending"}"${index === wizardStep ? ' aria-current="step"' : ""}><span aria-hidden="true">${index < wizardStep ? "✓" : index + 1}</span><div><strong>${title}</strong><small>${description}</small></div></li>`).join("")}</ol>
      <p class="rail-footer"><span class="signal" aria-hidden="true"></span> Works without an account</p>
    </aside>
    <div class="setup-main"><div class="setup-top"><span>Step ${wizardStep + 1} of ${setupSteps.length}</span><span class="prototype-label">Design prototype</span></div>
      <form id="setup-form" class="setup-form">
        <div class="setup-body">${setupContent()}</div>
        <footer class="setup-footer"><div>${!first ? '<button type="button" class="btn" data-action="setup-back">Back</button>' : '<button type="button" class="btn linklike" data-action="open-demo">Open sample workspace</button>'}</div>
        <button type="submit" id="setup-next" class="btn primary">${last ? "Open workspace" : first && chosenStart === "sample" ? "Explore workspace" : "Continue"}</button></footer>
      </form>
    </div></div>`;
  if (wizardStep === 2) syncGstinFields();
}

function syncGstinFields() {
  const gstin = onboarding.querySelector('[name="gstin"]');
  const required = setup.treatment !== "unregistered";
  gstin.pattern = "[0-9A-Za-z]{15}";
  gstin.disabled = !required;
  gstin.required = required;
  document.querySelector("#gstin-wrap").hidden = !required;
  document.querySelector("#irn-wrap").hidden = setup.treatment !== "regular";
  gstin.setCustomValidity(gstin.value && !gstin.value.startsWith(states[setup.state].slice(0, 2)) ? "GSTIN should begin with the selected state code." : "");
}

function changeSetupStep(step) {
  wizardStep = step;
  renderSetup();
  onboarding.querySelector("h1").focus();
  onboarding.scrollIntoView({ block: "start" });
}

function finishSetup(useDemo) {
  business = useDemo ? { ...demoBusiness } : { ...setup, name: setup.name.trim(), city: setup.city.trim(), prefix: setup.prefix.toUpperCase(), gstin: setup.treatment === "unregistered" ? "" : setup.gstin.trim().toUpperCase() };
  resetWorkspaceRecords(useDemo);
  onboarding.hidden = true;
  document.body.classList.remove("onboarding-active");
  document.querySelector("#workspace-sidebar").hidden = false;
  document.querySelector("#workspace-shell").hidden = false;
  document.querySelector("#workspace-mode").textContent = useDemo ? "Fictional records" : "Preview session";
  const skip = document.querySelector("#skip-link");
  skip.href = "#workspace";
  skip.textContent = "Skip to workspace";
  toast.hidden = true;
  openPage("home");
}

function stateName(state) { return states[state]?.slice(5) || ""; }

onboarding.addEventListener("submit", (event) => {
  event.preventDefault();
  if (wizardStep === 0 && chosenStart === "sample") return finishSetup(true);
  if (wizardStep === 1) {
    const invalid = ["name", "city"].find((key) => !setup[key].trim());
    if (invalid) {
      const field = event.target.elements[invalid];
      field.setCustomValidity("Enter a value to continue.");
      field.reportValidity();
      return;
    }
  }
  if (wizardStep === 0) setup.data = chosenStart === "import" ? "import" : "blank";
  if (wizardStep === setupSteps.length - 1) return finishSetup(false);
  changeSetupStep(wizardStep + 1);
});

function updateSetupField(field) {
  if (field.name in setup && field.type !== "radio") {
    field.setCustomValidity("");
    setup[field.name] = field.name === "gstin" ? field.value.toUpperCase() : field.value;
  }
  if (wizardStep === 2) syncGstinFields();
}

onboarding.addEventListener("input", (event) => updateSetupField(event.target));

onboarding.addEventListener("change", (event) => {
  const field = event.target;
  updateSetupField(field);
  if (field.name === "start") {
    chosenStart = field.value;
    document.querySelector("#setup-next").textContent = chosenStart === "sample" ? "Explore workspace" : "Continue";
  }
  if (field.name === "data") {
    setup.data = field.value;
    document.querySelector("#setup-import-wrap").hidden = setup.data !== "import";
  }
});

const importTemplates = {
  items: { title: "Items", columns: ["Name", "HSN", "Unit", "Rate", "GST"], example: "Cable clips,3926,Pack,60,18" },
  contacts: { title: "Customers & vendors", columns: ["Name", "Type", "State", "GSTIN"], example: "Sample Traders,Customer,West Bengal,19ABCDE1234F1Z5" },
  balances: { title: "Opening balances", columns: ["Account", "Debit", "Credit"], example: "Cash in hand,1500,0" },
  transactions: { title: "Sales & purchase transactions", columns: ["Date", "Type", "Number", "Total"], example: "2026-11-15,Invoice,SMP/001,1180" }
};

function importFeedback() {
  if (importError) return `<p class="import-warning" role="status">${esc(importError)}</p>`;
  if (!importPreview) return "";
  return `<div class="import-result" role="status"><b>${esc(importPreview.file)}</b><span>${countLabel(importPreview.rows, "row")} · ${esc(importPreview.detail)}</span></div>`;
}

function clearImport() {
  importRequest += 1;
  importPreview = null;
  importError = "";
  const picker = document.querySelector("#setup-import");
  if (picker) picker.value = "";
}

function csvRecords(text) {
  const records = [];
  let row = [];
  let value = "";
  let quoted = false;
  for (let index = 0; index < text.length; index += 1) {
    const char = text[index];
    if (char === '"' && quoted && text[index + 1] === '"') {
      value += char;
      index += 1;
    } else if (char === '"' && (quoted || !value)) {
      quoted = !quoted;
    } else if (!quoted && (char === "," || char === "\n" || char === "\r")) {
      row.push(value.trim());
      value = "";
      if (char === ",") continue;
      if (row.some(Boolean)) records.push(row);
      row = [];
      if (char === "\r" && text[index + 1] === "\n") index += 1;
    } else value += char;
  }
  if (quoted) throw Error("A quoted CSV field is incomplete.");
  row.push(value.trim());
  if (row.some(Boolean)) records.push(row);
  return records;
}

async function inspectImport(file) {
  const request = ++importRequest;
  const selectedTemplate = importTarget;
  importError = "";
  importPreview = null;
  if (!file) return;
  try {
    if (file.size > 3_000_000) throw Error("Use a sample file under 3 MB.");
    const content = await file.text();
    if (request !== importRequest) return;
    if (/\.json$/i.test(file.name)) {
      const data = JSON.parse(content);
      if (!data || data.format !== "ascel-bill-prototype" || data.version !== 1 || ![data.items, data.parties, data.invoices, data.drafts].every(Array.isArray)) {
        throw Error("Choose a JSON backup downloaded from this prototype.");
      }
      importPreview = { file: file.name, rows: data.items.length + data.parties.length + data.invoices.length + data.drafts.length, detail: "Sample backup structure recognized" };
    } else {
      const [header, ...rows] = csvRecords(content.replace(/^\uFEFF/, ""));
      const expected = importTemplates[selectedTemplate].columns;
      if (!header || expected.some((name) => !header.some((value) => value.toLowerCase() === name.toLowerCase()))) {
        throw Error(`Expected columns: ${expected.join(", ")}.`);
      }
      if (!rows.length || rows.some((row) => row.length !== header.length)) throw Error("Add at least one data row and check that all rows use the same columns.");
      importPreview = { file: file.name, rows: rows.length, detail: `Columns matched: ${expected.join(", ")}` };
    }
  } catch (error) {
    if (request !== importRequest) return;
    importError = error instanceof SyntaxError ? "Choose a valid JSON sample backup." : error.message;
  }
  const feedback = document.querySelector("#import-feedback");
  if (feedback) feedback.innerHTML = importFeedback();
}

const transactionExamples = {
  quotes: ["Quote", "QT", "Roy Electricals", 483800, "Accepted", "Use a quote to confirm quantities, pricing and payment terms before invoicing."],
  "sales-orders": ["Sales order", "SO", "Sen Industrial Supply", 151040, "Confirmed", "Confirm the order, then review fulfilment before invoicing."],
  "recurring-invoices": ["Schedule", "RI", "Roy Electricals", 94400, "Monthly", "Review the next draft date. Automatic sending is a connected feature."],
  retainers: ["Retainer", "RET", "Roy Electricals", 150000, "Applied in part", "Apply an advance against a later invoice after review."],
  "delivery-challans": ["Challan", "DC", "Sen Industrial Supply", 151040, "Ready to dispatch", "Check address, quantities and e-way bill requirements before stock moves."],
  "payments-received": ["Receipt", "REC", "Roy Electricals", 483800, "UPI recorded", "Review the payment method, date and related invoice."],
  "credit-notes": ["Credit note", "CN", "Roy Electricals", 9440, "Draft", "Link the correction to the original invoice and document the reason."],
  vendors: ["Vendor", "VEN", "Eastern Fasteners", 0, "Active", "Review supplier GST treatment and purchase contact details."],
  "purchase-orders": ["Purchase order", "PO", "Eastern Fasteners", 1440000, "Open", "Check items ordered, agreed price and delivery date."],
  receipts: ["Purchase receipt", "GRN", "Eastern Fasteners", 1440000, "Received", "Verify quantities and report exceptions before matching a bill."],
  bills: ["Supplier bill", "PB", "Eastern Fasteners", 1440000, "Due 30 Nov", "Match a vendor bill with received items and confirm input GST treatment."],
  expenses: ["Expense", "EXP", "Store transport", 95000, "Cash recorded", "Check the expense category, payment account and receipt."],
  "recurring-expenses": ["Expense schedule", "RE", "Store rent", 850000, "Monthly", "Review the period and estimated next charge before creating a draft."],
  "recurring-bills": ["Bill schedule", "RB", "Eastern Fasteners", 47200, "Monthly", "Set a schedule and review the next supplier-bill draft."],
  "payments-made": ["Vendor payment", "VP", "Metro Tools", 620000, "Bank transfer", "Review the matched supplier bill and transaction reference."],
  "vendor-credits": ["Vendor credit", "VC", "Eastern Fasteners", 23600, "Available", "Review the return reason and any credit applied to an open bill."],
  "bill-of-entry": ["Bill of entry", "BOE", "Sample consignment", 0, "Review", "Confirm customs reference and tax treatment with your accountant."],
  adjustments: ["Stock adjustment", "ADJ", "Insulation tape", 0, "Count corrected", "Review the physical count, adjustment quantity and reason."],
  journals: ["Journal", "JV", "Opening adjustment", 150000, "Draft", "Check that debits and credits balance before posting."],
  "recurring-journals": ["Journal schedule", "RJ", "Monthly accrual", 100000, "Monthly", "Review the date, accounts and balanced debit and credit entries."],
  projects: ["Project", "PRJ", "Electrical layout review", 225000, "Time unbilled", "Review tracked time and expenses before converting them to an invoice."]
};

const featureDetails = {
  "price-lists": [["Retail", "Use catalogue selling prices."], ["Wholesale", "Preview alternate rates by customer group."]],
  warehouses: [["Main counter", "Stock available for billing."], ["Back room", "Preview a transfer between two sample locations."]],
  "inventory-valuation": [["Quantity on hand", "Compare receipts, sales and adjustments."], ["Cost basis", "Review estimated value by item with your accountant."]],
  "bank-accounts": [["Cash in hand", "Review opening cash and manual shop receipts."], ["Current account", "Review deposits, transfers and vendor payments."]],
  "bank-statements": [["Choose a statement", "Preview date, description, debit, credit and balance columns."], ["Match transactions", "Review an imported line against purchases or receipts."]],
  reconciliation: [["Statement ending balance", "Confirm the date and balance for a period."], ["Unmatched transactions", "Review deposits, withdrawals and timing differences."]],
  "bank-feeds": [["Connect an account", "A supported provider and explicit authorisation are required."], ["Review imported activity", "Categorize transactions after they reach the workspace."]],
  gstr1: [["B2B, B2C & amendments", "Review invoice groups, notes and period corrections."], ["HSN & document summary", "Prepare outward-supply working papers for accountant review."]],
  gstr3b: [["Outward supply & liability", "Review taxable supplies and GST components."], ["Input tax credit & payment", "Check eligible credit with your accountant before filing."]],
  gstr2b: [["Import or fetch statement", "Choose a period and obtain GSTR-2B data."], ["Match with supplier bills", "Review matched, missing and differing amounts."]],
  ims: [["Inward supplies", "Review accepted, rejected and pending entries."], ["GSTN action history", "See where submitted decisions would be tracked."]],
  einvoicing: [["IRN queue", "Review unnumbered drafts blocked from issue in the offline simulation."], ["Portal connection", "A government IRP must issue the IRN and signed QR."]],
  eway: [["Dispatch readiness", "Review the shipment, transport details and applicability."], ["E-way bill reference", "Preview attaching a portal-issued number and validity to the dispatch."]],
  composition: [["Bill of supply", "Preview what a composition business would issue."], ["CMP-08 working papers", "Review the proposed quarterly summary with your accountant."]],
  "tax-settings": [["GST treatment", "Regular, composition and unregistered profiles are visible in setup."], ["Tax preferences", "Review rate groups, place of supply and reverse charge with your accountant."]],
  coa: [["Assets & liabilities", "Cash, receivables, stock, supplier balances and tax payable."], ["Income & expenses", "Sales, cost of goods and operating expenses."]],
  balances: [["Opening date", "See the date selected during setup."], ["Opening accounts", "Review cash, bank, outstanding customer and supplier balances before import."]],
  locking: [["Choose a closing date", "The proposed control prevents later edits to reviewed periods."], ["Review exceptions", "Unlocking would require authorisation and a reason."]],
  audit: [["Document activity", "Track issue, reprint and linked corrections by date."], ["Workspace changes", "Review when setup fields and preferences would be audited."]],
  approvals: [["Pending review", "Show the queue for draft purchases and journals."], ["Decision history", "Record approval, rejection and reason against each document."]],
  "sales-report": [["Sales by customer", "Compare customer totals and invoice counts."], ["Sales by item", "Compare quantities, taxable value and tax."]],
  "ar-aging": [["Current & overdue", "Group unpaid invoices by days until or past due."], ["Customer statement", "Review invoices, receipts and open amounts for one customer."]],
  "ap-aging": [["Current & overdue", "Group supplier bills by days past due."], ["Vendor statement", "Review bills, settlements and credits for one supplier."]],
  pnl: [["Income and cost of sales", "Review gross profit over a selected accounting period."], ["Operating expenses", "Review expense categories and net profit."]],
  "balance-sheet": [["Assets", "Review cash, receivables, stock and other assets."], ["Liabilities & equity", "Review payable, tax and owner-balance accounts."]],
  cashflow: [["Operating cash flow", "Trace collections and operating spending."], ["Other cash movements", "Review investing and financing flows separately."]],
  "trial-balance": [["Trial balance", "Review total debit and credit balance at period end."], ["General ledger", "Drill into proposed transaction-level entries for an account."]],
  documents: [["Receipt attachments", "Link supporting records to expenses and bills."], ["Document library", "Review access and retention for stored files."]],
  "customer-portal": [["Customer access", "Clients would view invoices and account balances online."], ["Quote approvals", "Preview where customers could review and accept a quote."]],
  "online-payments": [["Offline payment records", "Record cash, bank transfers or confirmed UPI receipts locally."], ["Payment gateways", "Connected providers would handle payment links, status and refunds."]],
  automation: [["Payment reminders", "Design due-date triggers and how recipients are selected."], ["Scheduled transactions", "Review drafts generated by configured billing schedules."]],
  users: [["Owner", "Manage setup, exports and workspace access."], ["Cashier & accountant", "Preview separate invoicing and reconciliation roles."]],
  preferences: [["Visible modules", "Review which tools a shop would enable."], ["Custom fields & tags", "Design additional transaction details for reports and documents."]],
  devices: [["Primary issuing device", "One issuing workstation is the proposed starting point."], ["Other devices", "Review portable backups and a future opt-in read-only companion."]]
};

const extraRecords = new Map();

function navigationView() {
  const current = moduleRoutes.get(page);
  const link = (id, text, icon = "", nested = false) =>
    `<button class="nav-button${nested ? " nav-child" : ""}" data-page="${id}"${id === page ? ' aria-current="page"' : ""}>${icon ? `<span class="nav-icon" aria-hidden="true">${icon}</span>` : ""}${text}</button>`;
  return `<div class="nav-top">${link("home", "Overview", "⌂")}${link("billing", "New invoice", "⊕")}</div>` + modules.map((group) => {
    const expanded = current.group === group.id && page !== "billing";
    return `<div class="nav-group${expanded ? " nav-open" : ""}">${link(group.id, group.title, group.icon)}
      ${expanded ? `<div class="nav-children">${group.entries.map(([id, title]) => link(id, title, "", true)).join("")}</div>` : ""}</div>`;
  }).join("");
}

function routeButton(id, label, primary = false) {
  return `<button class="btn${primary ? " primary" : ""}" data-page="${id}">${label || moduleRoutes.get(id).title}</button>`;
}

function moduleCard([id, title, description, mode]) {
  return `<button class="module-card" data-page="${id}"><span><strong>${title}</strong>${mode === "online" ? '<small class="badge amber">Online workflow</small>' : ""}</span><p>${description}</p></button>`;
}

function groupView(group) {
  return head(group.title, group.description, group.id === "sales" ? routeButton("billing", "＋ New invoice", true) : "") +
    `<div class="module-grid">${group.entries.map(moduleCard).join("")}</div>`;
}

function overviewShortcuts() {
  if (sampleMode) {
    const shortcuts = [["items", "Items & services"], ["customers", "Customers"], ["bills", "Supplier bills"], ["expenses", "Expenses"], ["reports", "GST sales register"], ["backup", "Backup & restore"]];
    return `<section class="card"><div class="card-head"><h2>Shortcuts</h2></div><div class="shortcut-grid card-body">${shortcuts.map(([id, label]) => routeButton(id, label)).join("")}</div></section>`;
  }
  const checklist = [
    ["items", "Add an item or service", items.length > 0],
    ["customers", "Add a customer", parties.length > 1],
    ["billing", "Create a first invoice", allIssued().length > 0],
    ["backup", "Export a workspace backup", Boolean(backupTime)]
  ];
  return `<section class="card"><div class="card-head"><h2>Get ready to bill</h2></div><ol class="billing-checklist card-body">${checklist.map(([id, label, complete]) =>
    `<li>${routeButton(id, label)}${complete ? '<span class="badge green">Done</span>' : ""}</li>`).join("")}</ol></section>`;
}

function homeView() {
  const records = allIssued();
  let sales = 0;
  let tax = 0;
  let unpaid = 0;
  for (const bill of records) {
    const calc = calculate(bill);
    sales += calc.gross;
    tax += calc.cgst + calc.sgst + calc.igst;
    if (bill.payment === "On credit") unpaid += calc.gross;
  }
  const recent = records.slice(0, 5).map((bill) => `<tr><td><strong class="number">${esc(bill.number)}</strong></td><td>${esc(partyById(bill.party).name)}</td><td>${dateText(bill.date)}</td><td><span class="badge ${bill.payment === "On credit" ? "amber" : "green"}">${bill.payment === "On credit" ? "On credit" : "Payment recorded"}</span></td><td class="number">${money(calculate(bill).gross)}</td><td><button class="btn" data-action="view-invoice" data-id="${esc(bill.number)}">View</button></td></tr>`).join("");
  const activity = new Map();
  for (const bill of records.slice().reverse()) activity.set(bill.date, (activity.get(bill.date) || 0) + calculate(bill).gross);
  const highest = Math.max(1, ...activity.values());
  return head("Overview", `${esc(business.name)} · FY 2026–27`, routeButton("billing", "＋ New invoice", true)) +
    `<div class="overview-hero"><div><span class="hero-kicker"><span class="signal" aria-hidden="true"></span> LOCAL WORKSPACE</span><h2>${sampleMode ? "Bengal Hardware is ready to explore" : "Your workspace is ready"}</h2><p>${sampleMode ? "Try a sale, review a purchase or open the GST working papers." : "Add an item and customer, then create your first sample invoice."}</p></div>
    <div class="hero-actions">${routeButton(sampleMode ? "sales" : "items", sampleMode ? "Browse sales" : "Add an item")}${routeButton("profile", "Business setup")}</div></div>
    <div class="metrics overview-metrics">${[["Invoiced sales", sales, "All issued invoices"], ["Open receivables", unpaid, "Credit sales in this session"], ["GST on sales", tax, "CGST, SGST / UTGST and IGST"]].map(([label, value, caption]) =>
      `<div class="card metric"><span>${label}</span><strong class="number">${money(value)}</strong><small>${caption}</small></div>`).join("")}</div>
    <div class="overview-grid">
      <section class="card"><div class="card-head"><h2>Sales activity</h2><span class="badge">${sampleMode ? "Sample data" : "This session"}</span></div>
      <div class="activity card-body">${activity.size ? [...activity].slice(-6).map(([date, value]) =>
        `<div class="activity-row"><span>${dateText(date)}</span><span class="bar-track" aria-hidden="true"><i style="--progress:${value / highest * 100}%"></i></span><strong class="number">${money(value)}</strong></div>`).join("") :
        '<p class="sub">Issued invoices will appear here.</p>'}</div></section>
      ${overviewShortcuts()}
    </div>
    <section class="card"><div class="card-head"><h2>Recent invoices</h2>${routeButton("invoices", "All invoices")}</div>
      ${recent ? table(["Invoice", "Customer", "Date", "Payment", "Total", "Action"], recent, "Recent invoices") : '<p class="empty">Your first issued invoice will appear here.</p>'}</section>`;
}

function propertyRows(rows) {
  return `<dl class="properties">${rows.map(([label, value]) => `<div><dt>${label}</dt><dd>${esc(value || "—")}</dd></div>`).join("")}</dl>`;
}

function profileView() {
  return head("Business profile", "Organization details selected in first-launch setup.", '<button class="btn" data-action="restart-setup">Preview setup again</button>') +
    `<div class="overview-grid">
    <section class="card"><div class="card-head"><h2>Business details</h2></div><div class="card-body">${propertyRows([["Business", business.name], ["Activity", business.type], ["City", business.city], ["State", stateName(business.state)], ["Address", business.address], ["Currency", "INR"], ["GST treatment", treatments[business.treatment]], ["GSTIN", business.gstin]])}</div></section>
    <section class="card"><div class="card-head"><h2>Billing & data</h2></div><div class="card-body">${propertyRows([["Prefix", business.prefix], ["Print layout", business.printer], ["Payment default", business.payment], ["Opening date", dateText(business.openingDate)], ["Opening cash", money(Math.round(Number(business.openingCash || 0) * 100))]])}</div></section></div>`;
}

function genericView(id, info) {
  if (id === "profile") return profileView();
  if (id === "numbering") return head(info.title, info.description) +
    `<section class="card"><div class="card-head"><h2>Invoice defaults</h2></div><div class="card-body">${propertyRows([["Next invoice number", billNumber()], ["Print layout", business.printer], ["Payment method", business.payment]])}${routeButton("billing", "Open invoice preview")}</div></section>`;
  if (id === "data-import") return head(info.title, info.description) +
    `<div class="two-cards"><section class="card"><h2>Preview migration</h2><p class="sub">Inspect a sample export and its expected columns.</p><button class="btn primary" data-action="standalone-import">Preview import</button></section>
    <section class="card"><h2>Opening balances</h2><p class="sub">Review the start date and opening accounts with your accountant.</p>${routeButton("balances")}</section></div>`;
  if (transactionExamples[id]) return transactionView(id, info, transactionExamples[id]);

  const details = featureDetails[id] || [];
  const usedDrafts = id === "einvoicing" ? draftRecords.filter(requiresIrn) : [];
  const related = {
    "gstr1": ["reports", "Open GST sales register"],
    "gstr3b": ["reports", "Open local tax totals"],
    "gstr2b": ["bills", "Open supplier bills"],
    "ims": ["gstr2b", "Open GSTR-2B preview"],
    "eway": ["delivery-challans", "Open delivery challans"],
    "balances": ["profile", "Open business profile"],
    "customer-portal": ["invoices", "Open invoices"],
    "online-payments": ["payments-received", "Open payments received"],
    "bank-feeds": ["bank-statements", "Open statement import preview"],
    "ar-aging": ["invoices", "Review invoices"],
    "ap-aging": ["bills", "Review supplier bills"]
  }[id];
  return head(info.title, info.description, related ? routeButton(...related) : "") +
    `<div class="preview-notice"><span class="badge ${info.mode === "online" ? "amber" : "green"}">${info.mode === "online" ? "Online workflow" : "Design preview"}</span><p>${info.mode === "online" ? "This screen shows where a connected service would be used. No account or portal is connected in the prototype." : "Review the proposed options for this area."}</p></div>
    ${usedDrafts.length ? `<section class="card module-drafts"><div class="card-head"><h2>Awaiting IRN</h2><span class="badge amber">${countLabel(usedDrafts.length, "draft")}</span></div>${table(["Customer", "Date", "Total", "Action"], usedDrafts.map((bill) =>
      `<tr><td>${esc(partyById(bill.party).name)}</td><td>${dateText(bill.date)}</td><td>${money(calculate(bill).gross)}</td><td><button class="btn" data-action="edit-draft" data-id="${esc(bill.id)}">Open draft</button></td></tr>`).join(""), "Drafts awaiting IRN")}</section>` : ""}
    <div class="module-grid">${details.map(([title, description], index) =>
      `<button class="module-card" data-action="workflow-detail" data-id="${id}" data-index="${index}"><span><strong>${title}</strong></span><p>${description}</p></button>`).join("")}</div>`;
}

function transactionView(id, info, example) {
  const [label, prefix, party, amount, status] = example;
  const fixture = sampleMode ? [{ number: `${prefix}/26-27/0001`, date: "2026-11-24", party, amount, status }] : [];
  const rows = [...(extraRecords.get(id) || []), ...fixture];
  return head(info.title, info.description, `<button class="btn primary" data-action="module-draft" data-id="${id}">＋ New ${label.toLowerCase()}</button>`) +
    `<div class="preview-notice"><span class="badge green">Design preview</span><p>Open an example or save a draft in this session to review the workflow.</p></div>
    <section class="card">${rows.length ? table(["Reference", "Party / description", "Date", "Status", "Amount", "Action"], rows.map((row, index) =>
      `<tr><td class="number"><strong>${esc(row.number)}</strong></td><td>${esc(row.party)}</td><td>${dateText(row.date)}</td><td><span class="badge ${row.status === "Draft" ? "amber" : "green"}">${row.status}</span></td><td class="number">${row.amount ? money(row.amount) : "—"}</td><td><button class="btn" data-action="module-record" data-id="${id}" data-index="${index}">View</button></td></tr>`).join(""), info.title) :
      '<p class="empty">No records yet. Create a draft to see it here.</p>'}</section>`;
}

function showModuleRecord(id, index) {
  const [label, prefix, party, amount, status, guidance] = transactionExamples[id];
  const rows = [...(extraRecords.get(id) || []), ...(sampleMode ? [{ number: `${prefix}/26-27/0001`, date: "2026-11-24", party, amount, status }] : [])];
  const row = rows[index];
  if (!row) return;
  const conversion = id === "quotes" && row.status === "Accepted";
  openDialog(esc(label), `<p class="sub">${guidance}</p>${propertyRows([["Reference", row.number], ["Party / description", row.party], ["Date", dateText(row.date)],
    ["Status", row.status], ["Amount", row.amount ? money(row.amount) : "—"]])}<div class="dialog-footer"><button class="btn" data-action="close">Close</button>${conversion ? '<button class="btn primary" data-action="convert-quote">Convert sample quote</button>' : ""}</div>`);
}

function showModuleDraft(id) {
  const [label] = transactionExamples[id];
  const isVendor = id === "vendors";
  openDialog(`New ${label.toLowerCase()}`, `<form id="module-draft-form" data-module="${id}"><div class="fields">
    <label class="field">${isVendor ? "Vendor name" : "Party or description"}<input name="party" required maxlength="90" autofocus></label>
    <label class="field">Date <input name="date" type="date" value="${demoDate}" required></label>
    ${isVendor ? "" : '<label class="field">Amount in ₹ <input name="amount" type="number" min="0" max="1000000000" step="0.01"></label>'}
    </div><p class="sub">The draft is available for this preview session.</p>
    <div class="dialog-footer"><button type="button" class="btn" data-action="close">Cancel</button><button class="btn primary" type="submit">Save draft</button></div></form>`);
}

document.querySelector("#dialog").addEventListener("submit", (event) => {
  if (event.target.id !== "module-draft-form") return;
  event.preventDefault();
  const id = event.target.dataset.module;
  const [, prefix] = transactionExamples[id];
  const values = new FormData(event.target);
  const party = String(values.get("party")).trim();
  if (!party) {
    event.target.elements.party.setCustomValidity("Enter a party or description.");
    event.target.elements.party.reportValidity();
    return;
  }
  const rows = extraRecords.get(id) || [];
  const serial = rows.length + (sampleMode ? 2 : 1);
  rows.unshift({ number: `${prefix}/26-27/${String(serial).padStart(4, "0")}`, party,
    amount: Math.round(Number(values.get("amount") || 0) * 100), date: values.get("date"), status: "Draft" });
  extraRecords.set(id, rows);
  dialog.close();
  render();
  showToast("Draft saved for this preview session.");
});

function showModuleSearch() {
  openDialog("Find a module", '<label class="field">Search modules <input id="module-query" type="search" autofocus placeholder="Try GST, bank, credit note or backup"></label><div class="pick-list search-results" id="module-results"></div>');
  showModuleMatches("");
  document.querySelector("#module-query").focus();
}

function showModuleMatches(term) {
  const query = term.trim().toLowerCase();
  const matches = [...moduleRoutes].filter(([, info]) =>
    `${info.title} ${info.description || ""}`.toLowerCase().includes(query));
  document.querySelector("#module-results").innerHTML = matches.length ? matches.map(([id, info]) =>
    `<button class="pick" data-action="search-open" data-id="${id}"><span><b>${info.title}</b><small class="item-meta">${info.group && info.group !== id ? moduleRoutes.get(info.group).title : "Workspace"}</small></span></button>`).join("") : '<p class="empty">No matching modules.</p>';
}

const experienceActions = {
  "setup-back": () => changeSetupStep(wizardStep - 1),
  "setup-edit": (button) => changeSetupStep(Number(button.dataset.id)),
  "open-demo": () => finishSetup(true),
  "sample-profile": () => {
    const { name, city, state, address, gstin, prefix, irn } = demoBusiness;
    Object.assign(setup, { name, city, state, address, gstin, prefix, irn });
    renderSetup();
    onboarding.querySelector('[name="name"]').focus();
  },
  "import-template": () => {
    const fixture = importTemplates[importTarget];
    download(`ascel-bill-sample-${importTarget}.csv`, fixture.columns.join(",") + "\r\n" + fixture.example + "\r\n", "text/csv;charset=utf-8");
  },
  "restart-setup": () => {
    openDialog("Return to first-launch setup?",
      '<p class="sub">Reopening setup starts a new preview workspace. Download a sample backup first if you want a copy of this session.</p><div class="dialog-footer"><button class="btn" data-action="close">Keep workspace</button><button class="btn primary" data-action="confirm-setup">Return to setup</button></div>');
  },
  "confirm-setup": () => {
    dialog.close();
    wizardStep = 0;
    clearImport();
    openSetup();
  },
  "standalone-import": () => { clearImport(); openDialog("Preview an import", importOptions()); },
  "module-search": showModuleSearch,
  "search-open": (button) => { dialog.close(); openPage(button.dataset.id); },
  "workflow-detail": (button) => {
    const id = button.dataset.id;
    const [title, description] = featureDetails[id][Number(button.dataset.index)];
    openDialog(title, `<p>${description}</p><p class="sub">${moduleRoutes.get(id).mode === "online" ? "Connection and submission are simulated." : "This option is included for workflow review."}</p><div class="dialog-footer"><button class="btn primary" data-action="close">Done</button></div>`);
  },
  "module-record": (button) => showModuleRecord(button.dataset.id, Number(button.dataset.index)),
  "module-draft": (button) => showModuleDraft(button.dataset.id),
  "convert-quote": () => {
    dialog.close();
    scenario = "local";
    draft = makeDraft();
    openPage("billing");
    showToast("Sample quote copied into a new invoice draft.");
  }
};

document.addEventListener("change", (event) => {
  if (event.target.id === "import-target") {
    importTarget = event.target.value;
    clearImport();
    document.querySelector("#import-feedback").innerHTML = "";
  }
  if (event.target.id === "setup-import") inspectImport(event.target.files[0]);
});

document.addEventListener("input", (event) => {
  if (event.target.id === "module-query") showModuleMatches(event.target.value);
  if (event.target.form?.id === "module-draft-form") event.target.setCustomValidity("");
});

document.addEventListener("keydown", (event) => {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k" && onboarding.hidden && !dialog.open) {
    event.preventDefault();
    showModuleSearch();
  }
});
