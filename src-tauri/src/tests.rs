use crate::{model::*, store};
use rusqlite::Connection;
use std::{fs, path::PathBuf};
use uuid::Uuid;

struct TestWorkspace {
    path: PathBuf,
    db: Connection,
}

impl TestWorkspace {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("ascel-bill-test-{}", Uuid::new_v4()));
        fs::create_dir(&path).unwrap();
        let db = store::open(&path.join("workspace.sqlite")).unwrap();
        Self { path, db }
    }
}
impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = self.db.execute_batch("PRAGMA journal_mode=DELETE;");
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn seller() -> Business {
    Business {
        name: "Fixture Hardware".into(),
        address: "Test Road, Kolkata".into(),
        state: "19".into(),
        gstin: "19ABCDE1234F1Z5".into(),
        prefix: "INV".into(),
        irn_required: false,
        allow_usd: false,
        print_settings: PrintSettings::default(),
    }
}

fn item(stock: Option<i64>) -> Item {
    Item {
        id: String::new(),
        name: "Sample cable".into(),
        hsn: "8544".into(),
        unit: "MTR".into(),
        price: 100_000,
        usd_price: None,
        rate: 1_800,
        stock,
    }
}

fn input(item_id: &str, state: &str) -> DraftInput {
    DraftInput {
        id: None,
        date: "2026-11-25".into(),
        customer_id: None,
        new_customer: None,
        template: None,
        currency: Currency::Inr,
        place: state.into(),
        delivery_address: String::new(),
        payment: "Credit".into(),
        reference: String::new(),
        lines: vec![LineInput {
            item_id: item_id.into(),
            quantity: 1_000,
            price: 100_000,
            discount: 0,
        }],
    }
}

fn buyer() -> Customer {
    Customer {
        id: String::new(),
        name: "Fixture Shop".into(),
        address: "Test Lane, Kolkata".into(),
        state: "19".into(),
        gstin: "19ABCDE1234F1Z5".into(),
        phone: String::new(),
        email: String::new(),
    }
}

#[test]
fn calculates_local_interstate_and_union_territory_components() {
    let mut w = TestWorkspace::new();
    store::setup(&mut w.db, seller()).unwrap();
    let item = store::add_item(&w.db, item(None)).unwrap();
    let local = store::review(&w.db, &input(&item.id, "19")).unwrap();
    assert_eq!(
        (
            local.totals.cgst,
            local.totals.local_gst,
            local.totals.igst,
            local.totals.total
        ),
        (9_000, 9_000, 0, 118_000)
    );
    let interstate = store::review(&w.db, &input(&item.id, "21")).unwrap();
    assert_eq!(
        (
            interstate.totals.cgst,
            interstate.totals.local_gst,
            interstate.totals.igst,
            interstate.totals.total
        ),
        (0, 0, 18_000, 118_000)
    );
    let mut ut = seller();
    ut.state = "04".into();
    let union = calculate(&input(&item.id, "04"), ut, None, vec![item]).unwrap();
    assert_eq!(union.local_tax_name, "UTGST");
}

#[test]
fn discounts_and_fractional_quantity_use_integer_paise() {
    let mut draft = input("item", "19");
    draft.lines[0].quantity = 1_125;
    draft.lines[0].price = 199;
    draft.lines[0].discount = 24;
    let bill = calculate(&draft, seller(), None, vec![item(None)]).unwrap();
    assert_eq!(
        (
            bill.totals.taxable,
            bill.totals.cgst,
            bill.totals.local_gst,
            bill.totals.total
        ),
        (200, 18, 18, 236)
    );
    draft.lines[0].discount = 999;
    assert!(calculate(&draft, seller(), None, vec![item(None)]).is_err());
}

#[test]
fn unregistered_seller_does_not_collect_gst() {
    let mut business = seller();
    business.gstin.clear();
    let bill = calculate(&input("item", "19"), business, None, vec![item(None)]).unwrap();
    assert_eq!(bill.totals.total, 100_000);
    assert_eq!(bill.lines[0].rate, 0);
}

#[test]
fn issue_is_idempotent_and_uses_a_fiscal_year_sequence() {
    let mut w = TestWorkspace::new();
    store::setup(&mut w.db, seller()).unwrap();
    let item = store::add_item(&w.db, item(Some(4_000))).unwrap();
    let mut draft = input(&item.id, "19");
    draft.date = "2026-03-31".into();
    let first = store::save_draft(&mut w.db, draft).unwrap().id.unwrap();
    assert_eq!(
        store::issue(&mut w.db, &first, None)
            .unwrap()
            .number
            .as_deref(),
        Some("INV/25-26/0001")
    );
    assert_eq!(
        store::issue(&mut w.db, &first, None)
            .unwrap()
            .number
            .as_deref(),
        Some("INV/25-26/0001")
    );
    let second = store::save_draft(&mut w.db, input(&item.id, "19"))
        .unwrap()
        .id
        .unwrap();
    assert_eq!(
        store::issue(&mut w.db, &second, None)
            .unwrap()
            .number
            .as_deref(),
        Some("INV/26-27/0001")
    );
    assert_eq!(
        store::items(&w.db, "", false).unwrap()[0].stock,
        Some(2_000)
    );
    assert_eq!(store::dashboard(&w.db).unwrap().invoice_count, 2);
    assert!(store::draft(&w.db, &first).is_err());
}

#[test]
fn failed_stock_check_rolls_back_invoice_stock_and_sequence() {
    let mut w = TestWorkspace::new();
    store::setup(&mut w.db, seller()).unwrap();
    let item = store::add_item(&w.db, item(Some(1_000))).unwrap();
    let mut draft = input(&item.id, "19");
    draft.lines.push(draft.lines[0].clone());
    let id = store::save_draft(&mut w.db, draft).unwrap().id.unwrap();
    assert!(store::issue(&mut w.db, &id, None)
        .unwrap_err()
        .contains("Insufficient stock"));
    assert_eq!(
        store::items(&w.db, "", false).unwrap()[0].stock,
        Some(1_000)
    );
    assert!(store::invoice(&w.db, &id).unwrap().is_none());
    let mut corrected = store::draft(&w.db, &id).unwrap();
    corrected.lines.pop();
    store::save_draft(&mut w.db, corrected).unwrap();
    assert_eq!(
        store::issue(&mut w.db, &id, None)
            .unwrap()
            .number
            .as_deref(),
        Some("INV/26-27/0001")
    );
}

#[test]
fn irn_gate_preserves_b2b_draft_and_permits_b2c() {
    let mut w = TestWorkspace::new();
    let mut business = seller();
    business.irn_required = true;
    store::setup(&mut w.db, business).unwrap();
    let item = store::add_item(&w.db, item(None)).unwrap();
    let customer = store::add_customer(&w.db, buyer()).unwrap();
    let mut draft = input(&item.id, "19");
    draft.customer_id = Some(customer.id);
    let id = store::save_draft(&mut w.db, draft).unwrap().id.unwrap();
    assert!(store::issue(&mut w.db, &id, None)
        .unwrap_err()
        .contains("needs an IRN"));
    assert!(store::draft(&w.db, &id).is_ok());
    let b2c = store::save_draft(&mut w.db, input(&item.id, "19"))
        .unwrap()
        .id
        .unwrap();
    assert!(store::issue(&mut w.db, &b2c, None)
        .unwrap()
        .number
        .is_some());
}

#[test]
fn requires_buyer_address_for_large_b2c_sale() {
    let mut draft = input("item", "19");
    draft.lines[0].quantity = 50_000;
    let bill = calculate(&draft, seller(), None, vec![item(None)]).unwrap();
    assert!(bill.blocked_reason.unwrap().contains("name and address"));
}

#[test]
fn drafts_and_issued_records_survive_reopen_and_backup_restore() {
    let mut w = TestWorkspace::new();
    store::setup(&mut w.db, seller()).unwrap();
    let item = store::add_item(&w.db, item(None)).unwrap();
    let id = store::save_draft(&mut w.db, input(&item.id, "19"))
        .unwrap()
        .id
        .unwrap();
    let issued = store::issue(&mut w.db, &id, None).unwrap();
    let draft_id = store::save_draft(&mut w.db, input(&item.id, "21"))
        .unwrap()
        .id
        .unwrap();
    let reopened = store::open(&w.path.join("workspace.sqlite")).unwrap();
    assert_eq!(store::draft(&reopened, &draft_id).unwrap().place, "21");
    assert_eq!(
        store::invoice(&reopened, &id).unwrap().unwrap().number,
        issued.number
    );
    let backup = w.path.join("test.ascelbill");
    store::snapshot(&reopened, &backup).unwrap();
    assert!(store::snapshot(&reopened, &backup).is_err());
    let mut other = TestWorkspace::new();
    let recovery = store::restore(&mut other.db, &backup, &other.path).unwrap();
    assert!(recovery.is_file());
    assert_eq!(store::dashboard(&other.db).unwrap().invoice_count, 1);
    assert_eq!(store::draft(&other.db, &draft_id).unwrap().place, "21");
    assert_eq!(
        store::issue(&mut other.db, &draft_id, None)
            .unwrap()
            .number
            .as_deref(),
        Some("INV/26-27/0002")
    );
    assert!(store::restore(&mut other.db, &backup, &other.path)
        .unwrap_err()
        .contains("remove issued invoices"));
    assert_eq!(store::dashboard(&other.db).unwrap().invoice_count, 2);
}

#[test]
fn sales_export_matches_issued_tax_and_escapes_spreadsheet_formulas() {
    let mut w = TestWorkspace::new();
    store::setup(&mut w.db, seller()).unwrap();
    let item = store::add_item(&w.db, item(None)).unwrap();
    let mut buyer = buyer();
    buyer.name = "=Example".into();
    let buyer = store::add_customer(&w.db, buyer).unwrap();
    let mut draft = input(&item.id, "21");
    draft.customer_id = Some(buyer.id);
    let id = store::save_draft(&mut w.db, draft).unwrap().id.unwrap();
    store::issue(&mut w.db, &id, None).unwrap();
    let report = w.path.join("sales.csv");
    assert_eq!(
        store::export_csv(&w.db, "2026-11", &report, Currency::Inr).unwrap(),
        1
    );
    let text = fs::read_to_string(&report).unwrap();
    assert!(text.contains("'=Example") && text.contains("180.00") && text.contains("1180.00"));
    assert_eq!(
        store::export_csv(&w.db, "2026-10", &w.path.join("empty.csv"), Currency::Inr).unwrap(),
        0
    );
}

#[test]
fn direct_customer_and_template_are_committed_with_the_invoice() {
    let mut w = TestWorkspace::new();
    store::setup(&mut w.db, seller()).unwrap();
    let settings = PrintSettings {
        template: InvoiceTemplate::LaserA5,
        accent: Accent::Black,
        footer: "Thank you".into(),
        show_contacts: true,
        ..PrintSettings::default()
    };
    store::save_print_settings(&mut w.db, settings.clone()).unwrap();
    let item = store::add_item(&w.db, item(Some(1_000))).unwrap();
    let mut contact = buyer();
    contact.phone = "09876 543210".into();
    contact.email = "BUYER@Example.com".into();
    let mut draft = input(&item.id, "19");
    draft.new_customer = Some(contact);
    draft.lines[0].quantity = 2_000;
    let draft = store::save_draft(&mut w.db, draft).unwrap();
    let id = draft.id.as_deref().unwrap();

    let preview = store::review(&w.db, &draft).unwrap();
    assert_eq!(preview.template, Some(InvoiceTemplate::LaserA5));
    assert_eq!(preview.print_settings.as_ref(), Some(&settings));
    assert!(preview.customer.unwrap().id.is_empty());
    assert!(store::customers(&w.db, "").unwrap().is_empty());
    assert!(store::issue(&mut w.db, id, Some(InvoiceTemplate::LaserA6))
        .unwrap_err()
        .contains("Insufficient stock"));
    assert!(store::customers(&w.db, "").unwrap().is_empty());

    let mut corrected = store::draft(&w.db, id).unwrap();
    corrected.lines[0].quantity = 1_000;
    store::save_draft(&mut w.db, corrected).unwrap();
    let invoice = store::issue(&mut w.db, id, Some(InvoiceTemplate::LaserA6)).unwrap();
    assert_eq!(invoice.template, Some(InvoiceTemplate::LaserA6));
    assert_eq!(invoice.number.as_deref(), Some("INV/26-27/0001"));
    let customer = invoice.customer.unwrap();
    assert_eq!(customer.phone, "+919876543210");
    assert_eq!(customer.email, "buyer@example.com");
    assert_eq!(
        store::customers(&w.db, "(98765) 43210").unwrap()[0].id,
        customer.id
    );
    assert_eq!(
        store::customers(&w.db, "BUYER@ex").unwrap()[0].id,
        customer.id
    );
    store::save_print_settings(&mut w.db, PrintSettings::default()).unwrap();
    let reopened = store::invoice(&w.db, id).unwrap().unwrap();
    assert_eq!(reopened.template, Some(InvoiceTemplate::LaserA6));
    assert_eq!(
        reopened.print_settings,
        Some(PrintSettings {
            template: InvoiceTemplate::LaserA6,
            ..settings
        })
    );
    assert_eq!(
        store::issue(&mut w.db, id, None)
            .unwrap()
            .customer
            .unwrap()
            .id,
        customer.id
    );
    assert_eq!(store::customers(&w.db, "").unwrap().len(), 1);
    assert_eq!(invoice.seller.print_settings, PrintSettings::default());
}

#[test]
fn direct_sale_reuses_a_matching_contact_and_rejects_conflicts() {
    let mut w = TestWorkspace::new();
    store::setup(&mut w.db, seller()).unwrap();
    let mut contact = buyer();
    contact.phone = "+91 98765-43210".into();
    contact.email = "buyer@example.com".into();
    let existing = store::add_customer(&w.db, contact.clone()).unwrap();
    let item = store::add_item(&w.db, item(None)).unwrap();
    let mut draft = input(&item.id, "19");
    contact.phone = "9876543210".into();
    contact.email = "Buyer@Example.COM".into();
    draft.new_customer = Some(contact.clone());
    let saved = store::save_draft(&mut w.db, draft).unwrap();
    assert_eq!(
        store::review(&w.db, &saved).unwrap().customer.unwrap().id,
        existing.id
    );
    let id = saved.id.unwrap();
    let result = store::issue(&mut w.db, &id, None).unwrap();
    assert_eq!(result.customer.unwrap().id, existing.id);
    assert_eq!(store::customers(&w.db, "").unwrap().len(), 1);

    let mut draft = input(&item.id, "19");
    contact.name = "Someone else".into();
    draft.new_customer = Some(contact);
    assert!(store::review(&w.db, &draft)
        .unwrap_err()
        .contains("Select the existing customer"));
}

#[test]
fn csv_preview_and_reimport_handle_quotes_and_duplicate_contacts() {
    let mut w = TestWorkspace::new();
    store::setup(&mut w.db, seller()).unwrap();
    let csv = "\u{feff}name,phone,email,state,address,gstin\r\n\"Buyer, One\",98765 43210,ONE@example.org,West Bengal,\"1 Road, Kolkata\",\r\nBuyer Duplicate,+919876543210,,19,,\r\nBad row,123,,19,,\r\nBuyer Two,,two@example.org,21,,\r\n";
    let preview = crate::contacts::import_csv(&mut w.db, csv, false).unwrap();
    assert_eq!(
        (
            preview.ready,
            preview.imported,
            preview.duplicates,
            preview.invalid
        ),
        (2, 0, 1, 1)
    );
    assert!(store::customers(&w.db, "").unwrap().is_empty());
    let applied = crate::contacts::import_csv(&mut w.db, csv, true).unwrap();
    assert_eq!(
        (applied.imported, applied.duplicates, applied.invalid),
        (2, 1, 1)
    );
    assert_eq!(store::customers(&w.db, "").unwrap().len(), 2);
    let one = store::customers(&w.db, "one@example")
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(one.name, "Buyer, One");
    assert_eq!(one.address, "1 Road, Kolkata");
    let again = crate::contacts::import_csv(&mut w.db, csv, true).unwrap();
    assert_eq!((again.imported, again.duplicates, again.invalid), (0, 3, 1));
}

#[test]
fn legacy_workspace_and_customer_json_open_with_print_defaults() {
    let w = TestWorkspace::new();
    w.db.execute("INSERT INTO business(id,data) VALUES(1,?)", [r#"{"name":"Old Shop","address":"Old Road","state":"19","gstin":"","prefix":"OLD","irnRequired":false}"#]).unwrap();
    w.db.execute(
        "INSERT INTO customers(id,name,data) VALUES('legacy','Old Customer',?)",
        [r#"{"id":"legacy","name":"Old Customer","address":"","state":"19","gstin":""}"#],
    )
    .unwrap();
    w.db.execute_batch("DROP INDEX customer_phone; DROP INDEX customer_email;")
        .unwrap();
    let reopened = store::open(&w.path.join("workspace.sqlite")).unwrap();
    assert_eq!(
        store::business(&reopened).unwrap().unwrap().print_settings,
        PrintSettings::default()
    );
    assert!(store::selected_customer(&reopened, "legacy")
        .unwrap()
        .phone
        .is_empty());
    let indexes: i64 = reopened
        .query_row(
            "SELECT count(*) FROM sqlite_schema WHERE name IN ('customer_phone','customer_email')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(indexes, 2);
}
