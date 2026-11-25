use crate::{
    model::{Business, Currency, DraftInput, Item, LineInput, PrintSettings, StockMode},
    overview, store,
};
use rusqlite::Connection;
use serde_json::{json, Value};
use std::{fs, path::PathBuf};
use uuid::Uuid;

fn workspace(gstin: &str) -> Connection {
    let mut db = Connection::open_in_memory().unwrap();
    db.execute_batch(include_str!("schema.sql")).unwrap();
    db.execute_batch("PRAGMA foreign_keys=ON; PRAGMA trusted_schema=OFF;")
        .unwrap();
    store::setup(
        &mut db,
        Business {
            name: "Fixture Hardware".into(),
            address: "Test Lane, Kolkata".into(),
            state: "19".into(),
            gstin: gstin.into(),
            prefix: "INV".into(),
            irn_required: false,
            allow_usd: false,
            print_settings: PrintSettings::default(),
        },
    )
    .unwrap();
    db
}

fn add_item(db: &Connection, stock: Option<i64>) -> Item {
    store::add_item(
        db,
        Item {
            id: String::new(),
            name: "Measuring tape".into(),
            hsn: "9017".into(),
            unit: "PCS".into(),
            price: 56_789,
            usd_price: Some(1_425),
            rate: 1_800,
            stock,
        },
    )
    .unwrap()
}

fn draft(item: &Item, currency: Currency) -> DraftInput {
    DraftInput {
        id: None,
        date: "2026-11-25".into(),
        customer_id: None,
        new_customer: None,
        template: None,
        currency,
        place: "19".into(),
        delivery_address: String::new(),
        payment: "Credit".into(),
        reference: String::new(),
        lines: vec![LineInput {
            item_id: item.id.clone(),
            quantity: 1_000,
            price: match currency {
                Currency::Inr => item.price,
                Currency::Usd => item.usd_price.unwrap(),
            },
            discount: 0,
        }],
    }
}

struct ExportDirectory(PathBuf);

impl ExportDirectory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("ascel-export-test-{}", Uuid::new_v4()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for ExportDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn older_workspace_records_default_to_inr_and_leave_usd_disabled() {
    let mut db = workspace("");
    let item = add_item(&db, None);
    let id = store::save_draft(&mut db, draft(&item, Currency::Inr))
        .unwrap()
        .id
        .unwrap();
    db.execute_batch(
        "UPDATE business SET data=json_remove(data,'$.allowUsd');
         UPDATE items SET data=json_remove(data,'$.usdPrice');
         UPDATE drafts SET data=json_remove(data,'$.currency');",
    )
    .unwrap();

    assert!(!store::business(&db).unwrap().unwrap().allow_usd);
    assert_eq!(store::items(&db, "").unwrap()[0].usd_price, None);
    assert_eq!(store::draft(&db, &id).unwrap().currency, Currency::Inr);
    assert_eq!(
        store::register(&db, "draft", "", 0).unwrap()[0].currency,
        Currency::Inr
    );
    let issued = store::issue(&mut db, &id, None).unwrap();
    db.execute(
        "UPDATE invoices SET data=json_remove(data,'$.currency','$.seller.allowUsd') WHERE id=?",
        [&id],
    )
    .unwrap();

    let legacy = store::invoice(&db, &id).unwrap().unwrap();
    assert_eq!(legacy.currency, Currency::Inr);
    assert_eq!(legacy.number, issued.number);
    assert_eq!(legacy.totals.total, 56_789);
    assert_eq!(
        store::register(&db, "issued", "", 0).unwrap()[0].currency,
        Currency::Inr
    );
}

#[test]
fn usd_is_opt_in_and_available_only_for_unregistered_businesses() {
    let mut db = workspace("19ABCDE1234F1Z5");
    let item = add_item(&db, None);
    assert!(store::set_usd_option(&mut db, true).is_err());
    assert!(!store::business(&db).unwrap().unwrap().allow_usd);
    assert!(store::review(&db, &draft(&item, Currency::Usd)).is_err());

    let mut db = workspace("");
    let item = add_item(&db, Some(2_000));
    let usd = draft(&item, Currency::Usd);
    assert!(store::review(&db, &usd).is_err());
    assert!(store::set_usd_option(&mut db, true).unwrap().allow_usd);
    let id = store::save_draft(&mut db, usd).unwrap().id.unwrap();
    let issued = store::issue(&mut db, &id, None).unwrap();
    assert_eq!(issued.currency, Currency::Usd);
    assert_eq!(issued.payment, "Credit");
    assert_eq!(issued.lines[0].rate, 0);
    assert_eq!(issued.totals.taxable, 1_425);
    assert_eq!(issued.totals.total, 1_425);
    assert_eq!(
        (
            issued.totals.cgst,
            issued.totals.local_gst,
            issued.totals.igst
        ),
        (0, 0, 0)
    );
    assert_eq!(store::items(&db, "").unwrap()[0].stock, Some(1_000));

    assert!(!store::set_usd_option(&mut db, false).unwrap().allow_usd);
    assert!(store::review(&db, &draft(&item, Currency::Usd)).is_err());
    assert_eq!(
        store::invoice(&db, &id).unwrap().unwrap().currency,
        Currency::Usd
    );
    assert_eq!(
        store::issue(&mut db, &id, None).unwrap().number,
        issued.number
    );
}

#[test]
fn register_overview_and_sales_exports_keep_currency_totals_separate() {
    let mut db = workspace("");
    let item = add_item(&db, None);
    store::set_usd_option(&mut db, true).unwrap();
    let inr = store::save_draft(&mut db, draft(&item, Currency::Inr))
        .unwrap()
        .id
        .unwrap();
    let usd = store::save_draft(&mut db, draft(&item, Currency::Usd))
        .unwrap()
        .id
        .unwrap();
    store::issue(&mut db, &inr, None).unwrap();
    store::issue(&mut db, &usd, None).unwrap();
    db.execute(
        "UPDATE invoices SET data=json_remove(data,'$.currency') WHERE id=?",
        [&inr],
    )
    .unwrap();

    let register = store::register(&db, "issued", "", 0).unwrap();
    assert_eq!(register.len(), 2);
    let summary = serde_json::to_value(overview::load(&db, "2026-11-25").unwrap()).unwrap();
    let directory = ExportDirectory::new();
    for (id, currency, total, display_total) in [
        (inr, Currency::Inr, 56_789, "567.89"),
        (usd, Currency::Usd, 1_425, "14.25"),
    ] {
        let row = register.iter().find(|row| row.id == id).unwrap();
        assert_eq!(row.currency, currency);
        assert_eq!(row.totals.as_ref().unwrap().total, total);
        let sales = &summary["sales"][currency.code()];
        assert_eq!(sales["invoiceCount"], 1);
        assert_eq!(sales["total"], total);
        assert_eq!(sales["creditSales"], total);

        let file = directory.0.join(format!("{}.csv", currency.code()));
        assert_eq!(
            store::export_csv(&db, "2026-11", &file, currency).unwrap(),
            1
        );
        let mut csv = csv::Reader::from_path(&file).unwrap();
        let header = csv.headers().unwrap();
        assert_eq!(
            header.iter().last().unwrap(),
            format!("Total {}", currency.code())
        );
        let records: Vec<_> = csv
            .records()
            .collect::<std::result::Result<_, _>>()
            .unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].get(0), row.number.as_deref());
        assert_eq!(records[0].iter().last(), Some(display_total));
    }
}

fn stock_audit(db: &Connection) -> Vec<(String, Value)> {
    let mut query = db
        .prepare("SELECT action, reference FROM audit WHERE action LIKE 'stock_%' ORDER BY id")
        .unwrap();
    let audit_entries = query
        .query_map([], |row| {
            let reference: String = row.get(1)?;
            Ok((row.get(0)?, serde_json::from_str(&reference).unwrap()))
        })
        .unwrap();
    audit_entries.map(|row| row.unwrap()).collect()
}

#[test]
fn receiving_and_counting_stock_preserve_audit_and_reject_overdrafts() {
    let mut db = workspace("");
    let item = add_item(&db, None);
    let received =
        store::update_stock(&mut db, &item.id, StockMode::Receive, 2_500, " Lot 7 ").unwrap();
    assert_eq!(received.stock, Some(2_500));
    assert_eq!(
        store::update_stock(&mut db, &item.id, StockMode::Receive, 750, "Extra case")
            .unwrap()
            .stock,
        Some(3_250)
    );
    let counted =
        store::update_stock(&mut db, &item.id, StockMode::Count, 0, "Shelf count").unwrap();
    assert_eq!(counted.stock, Some(0));

    for (mode, quantity) in [
        (StockMode::Receive, 0),
        (StockMode::Receive, -1),
        (StockMode::Receive, 1_000_000_001),
        (StockMode::Count, -1),
        (StockMode::Count, 1_000_000_001),
    ] {
        assert!(store::update_stock(&mut db, &item.id, mode, quantity, "Rejected").is_err());
    }

    let id = store::save_draft(&mut db, draft(&item, Currency::Inr))
        .unwrap()
        .id
        .unwrap();
    assert!(store::issue(&mut db, &id, None).is_err());
    assert!(store::invoice(&db, &id).unwrap().is_none());
    assert!(store::draft(&db, &id).is_ok());
    assert_eq!(store::items(&db, "").unwrap()[0].stock, Some(0));
    assert_eq!(
        stock_audit(&db),
        vec![
            (
                "stock_received".into(),
                json!({"itemId": item.id, "previous": null, "stock": 2500, "note": "Lot 7"})
            ),
            (
                "stock_received".into(),
                json!({"itemId": item.id, "previous": 2500, "stock": 3250, "note": "Extra case"})
            ),
            (
                "stock_counted".into(),
                json!({"itemId": item.id, "previous": 3250, "stock": 0, "note": "Shelf count"})
            )
        ]
    );
}

#[test]
fn stock_overflow_or_audit_write_failure_leaves_the_previous_count() {
    let mut db = workspace("");
    let item = add_item(&db, Some(1_000_000_000));
    assert!(store::update_stock(&mut db, &item.id, StockMode::Receive, 1, "Overflow").is_err());
    db.execute_batch(
        "CREATE TRIGGER failed_stock_audit BEFORE INSERT ON audit
         WHEN NEW.action = 'stock_counted' BEGIN SELECT RAISE(ABORT, 'test audit failure'); END;",
    )
    .unwrap();
    assert!(store::update_stock(&mut db, &item.id, StockMode::Count, 1, "Physical stock").is_err());
    assert_eq!(store::items(&db, "").unwrap()[0].stock, item.stock);
    assert!(stock_audit(&db).is_empty());
}
