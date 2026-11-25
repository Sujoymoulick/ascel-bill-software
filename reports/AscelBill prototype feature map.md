# Ascel Bill prototype feature map

**Design review · 25 November 2026**

Open the [clickable prototype](../prototype/index.html). It starts with setup every time you open a fresh page. Choose **Open sample workspace** to explore the full example right away, or use the five-step setup to test a first launch. Ascel Bill keeps its own green desktop UI and uses Zoho's documented India workflows as the feature reference. [Zoho Books India getting started](https://www.zoho.com/in/books/help/getting-started/zoho-books.html) · [Zoho Invoice India](https://www.zoho.com/in/invoice/)

## First launch

| Step | What to review |
| --- | --- |
| 1. Getting started | Create a business, preview a migration, or explore a fictional hardware shop. |
| 2. Business profile | Business name, goods/services, Indian state or union territory, city and address. India and INR are the current design defaults. |
| 3. GST and billing | Regular GST, composition and unregistered profiles. GSTIN format and matching state code, e-invoicing applicability for regular GST, document prefix, A4 or 80 mm layout preview and payment defaults. |
| 4. Opening data | Pick an opening date, enter optional cash, or inspect a sample CSV or JSON file. CSV examples cover items, contacts, opening balances and transactions. |
| 5. Review and launch | Edit a prior step, confirm settings, and open the local overview. A blank workspace guides the operator through items, customers, the first invoice and backup. |

The detailed setup follows Zoho Books' sequence of organisation and tax settings, opening balances, and master/transaction imports. The Ascel Bill migration path checks sample file structure and leaves the new workspace blank so reviewers can test setup safely. [Zoho Books settings](https://www.zoho.com/in/books/help/settings/) · [GST settings](https://www.zoho.com/in/books/help/settings/taxes.html) · [Migration order](https://www.zoho.com/in/books/kb/general/migration-import-data.html)

## Workspace options

There are 63 options across nine sections, plus Overview and the New invoice shortcut. Search by module name or keyword from the top bar, with Ctrl+K on Windows or Cmd+K on macOS.

| Area | Options represented | Interaction in this prototype |
| --- | --- | --- |
| Sales | Customers, quotes, sales orders, invoices, recurring and retainer invoices, delivery challans, receipts, credit notes | Add a customer, convert an accepted sample quote, edit lines, save and reopen drafts, issue a sample invoice and open its print view. Other sales records have examples and limited draft forms. |
| Purchases | Vendors, purchase orders and receipts, bills, expenses, recurring schedules, payments made, vendor credits, bills of entry | Review sample records and create session-only drafts. |
| Items and stock | Items and services, price lists, adjustments, stock locations and transfers, valuation | Add a catalogue item and use it on a sample invoice. Review stock workflow previews. |
| Banking | Bank and cash accounts, statement import, reconciliation, bank feeds | Review workflow steps and connection states. |
| GST and compliance | GST sales register, GSTR-1/1A, GSTR-3B, GSTR-2B, IMS, IRN, e-way bills, composition/CMP-08, tax preferences | Export a sample sales CSV. Try local and interstate GST and save an IRN-blocked draft to the queue. Review connected and period-end workflow previews. |
| Accountant | Chart of accounts, opening balances, journals, recurring journals, period locking, audit trail, approvals | Review workflow steps. Create session-only journal drafts for design review. |
| Reports | Sales breakdowns, receivables and payables ageing, profit and loss, balance sheet, cash flow, trial balance and general ledger | Review the proposed report menu and detail views. Dashboard sales metrics use the in-session sample invoices. |
| Projects and tools | Projects and timesheets, documents, customer portal, online payments, automation and reminders | Review local or connected workflow previews. Create a session-only project record. |
| Settings | Business profile, invoice numbering and templates, users and roles, preferences and custom fields, import, backup and restore, devices and sync | Review setup choices, inspect a sample file, and export and inspect a JSON backup. Reopen first-launch setup from Business profile. |

Feature availability varies across Zoho Invoice and Zoho Books plans. The catalogue above captures the main documented India billing and accounting workflows for design review. The [production proposal](AscelBill%20desktop%20product%20plan.md) defines the narrower first release separately. [Zoho Invoice features](https://www.zoho.com/in/invoice/features/) · [Zoho Books India features](https://www.zoho.com/in/books/accounting-software-features/) · [Zoho Books India plan comparison](https://www.zoho.com/in/books/pricing/)

## Local and connected workflows

The prototype runs from local files with fictional, session-only records. Core billing, document preview, sales CSV and backup export can be exercised locally. The GST filing, IRP, e-way bill, bank feed, payment gateway, customer portal and device sync screens show where connected actions would happen. A sample IRN-required sale can be saved as a draft and has no issue action. An authorized IRP connection and signed QR are part of a live e-invoicing flow. [Zoho e-invoicing setup](https://www.zoho.com/in/books/help/e-invoicing/set-up.html) · [Zoho GSTR-1 filing](https://www.zoho.com/in/books/help/gst/gstr1-filing.html) · [Zoho IMS](https://www.zoho.com/in/books/help/gst/ims.html)

### Suggested review sequence

1. Run onboarding with **Fill with fictional details** and check the confirmation screen.
2. Open the sample workspace and convert an accepted quote from **Sales → Quotes**.
3. In the invoice editor, compare **Local sale**, **Interstate sale** and **IRN required**. Open the GST section to see the saved IRN draft.
4. Open **Purchases → Supplier bills**, save a sample draft, then try **Settings → Backup & restore**.
