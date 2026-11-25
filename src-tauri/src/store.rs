pub use crate::contacts::add_customer;
use crate::{contacts, model::*};
use rusqlite::{
    backup::Backup, params, Connection, OpenFlags, OptionalExtension, TransactionBehavior,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};
use uuid::Uuid;

const APP_ID: i64 = 0x4153424C;
const VERSION: i64 = 1;
const PAGE_SIZE: i64 = 60;

pub(crate) fn db_error(error: impl std::fmt::Display) -> String {
    format!("Local data error: {error}")
}

pub(crate) fn json(value: &impl Serialize) -> Result<String> {
    serde_json::to_string(value).map_err(db_error)
}

pub(crate) fn parse<T: DeserializeOwned>(value: &str) -> Result<T> {
    serde_json::from_str(value).map_err(db_error)
}

pub fn open(path: &Path) -> Result<Connection> {
    let mut db = Connection::open(path).map_err(db_error)?;
    db.busy_timeout(Duration::from_secs(3)).map_err(db_error)?;
    db.execute_batch("PRAGMA foreign_keys=ON; PRAGMA trusted_schema=OFF;")
        .map_err(db_error)?;
    let version: i64 = db
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(db_error)?;
    if version == 0 {
        let transaction = db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        transaction
            .execute_batch(include_str!("schema.sql"))
            .map_err(db_error)?;
        transaction
            .pragma_update(None, "application_id", APP_ID)
            .map_err(db_error)?;
        transaction
            .pragma_update(None, "user_version", VERSION)
            .map_err(db_error)?;
        transaction.commit().map_err(db_error)?;
    }
    verify_schema(&db)?;
    db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;")
        .map_err(db_error)?;
    contacts::ensure_indexes(&db)?;
    Ok(db)
}

fn verify_schema(db: &Connection) -> Result<()> {
    let identity: i64 = db
        .pragma_query_value(None, "application_id", |row| row.get(0))
        .map_err(db_error)?;
    let version: i64 = db
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(db_error)?;
    if identity != APP_ID || version != VERSION {
        return Err("Choose an Ascel Bill database from a compatible version.".into());
    }
    Ok(())
}

pub fn business(db: &Connection) -> Result<Option<Business>> {
    db.query_row("SELECT data FROM business WHERE id=1", [], |row| {
        row.get::<_, String>(0)
    })
    .optional()
    .map_err(db_error)?
    .map(|data| parse(&data))
    .transpose()
}

pub fn setup(db: &mut Connection, data: Business) -> Result<Business> {
    validate_business(&data)?;
    let tx = db
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(db_error)?;
    if business(&tx)?.is_some() {
        return Err("This workstation already has a business.".into());
    }
    tx.execute("INSERT INTO business(id,data) VALUES(1,?)", [json(&data)?])
        .map_err(db_error)?;
    tx.execute(
        "INSERT INTO audit(action,reference) VALUES('business_created',?)",
        [&data.name],
    )
    .map_err(db_error)?;
    tx.commit().map_err(db_error)?;
    Ok(data)
}

pub fn update_business(
    db: &mut Connection,
    name: String,
    address: String,
    irn_required: bool,
) -> Result<Business> {
    let tx = db
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(db_error)?;
    let mut data = require_business(&tx)?;
    data.name = name;
    data.address = address;
    data.irn_required = irn_required;
    validate_business(&data)?;
    tx.execute("UPDATE business SET data=? WHERE id=1", [json(&data)?])
        .map_err(db_error)?;
    tx.execute(
        "INSERT INTO audit(action,reference) VALUES('business_updated',?)",
        [&data.name],
    )
    .map_err(db_error)?;
    tx.commit().map_err(db_error)?;
    Ok(data)
}

pub fn save_print_settings(db: &mut Connection, settings: PrintSettings) -> Result<Business> {
    validate_print_settings(&settings)?;
    let tx = db
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(db_error)?;
    let mut business = require_business(&tx)?;
    business.print_settings = settings;
    tx.execute("UPDATE business SET data=? WHERE id=1", [json(&business)?])
        .map_err(db_error)?;
    tx.commit().map_err(db_error)?;
    Ok(business)
}

pub fn set_usd_option(db: &mut Connection, allowed: bool) -> Result<Business> {
    let tx = db
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(db_error)?;
    let mut data = require_business(&tx)?;
    data.allow_usd = allowed;
    validate_business(&data)?;
    tx.execute("UPDATE business SET data=? WHERE id=1", [json(&data)?])
        .map_err(db_error)?;
    tx.commit().map_err(db_error)?;
    Ok(data)
}

pub fn update_stock(
    db: &mut Connection,
    id: &str,
    mode: StockMode,
    quantity: i64,
    note: &str,
) -> Result<Item> {
    text(note, "the stock update note", 180, false)?;
    if !(0..=1_000_000_000).contains(&quantity)
        || (matches!(mode, StockMode::Receive) && quantity == 0)
    {
        return Err("Enter a stock quantity within the allowed range.".into());
    }
    let tx = db
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(db_error)?;
    let mut item = selected_items(&tx, &[id.to_owned()])?
        .pop()
        .ok_or("That item could not be found.")?;
    let previous = item.stock;
    let (stock, action) = match mode {
        StockMode::Receive => (previous.unwrap_or_default() + quantity, "stock_received"),
        StockMode::Count => (quantity, "stock_counted"),
    };
    if stock > 1_000_000_000 {
        return Err("Enter a stock quantity within the allowed range.".into());
    }
    tx.execute("UPDATE items SET stock=? WHERE id=?", params![stock, id])
        .map_err(db_error)?;
    let reference =
        serde_json::json!({"itemId":id,"previous":previous,"stock":stock,"note":note.trim()});
    tx.execute(
        "INSERT INTO audit(action, reference) VALUES(?,?)",
        params![action, reference.to_string()],
    )
    .map_err(db_error)?;
    tx.commit().map_err(db_error)?;
    item.stock = Some(stock);
    Ok(item)
}

pub fn add_item(db: &Connection, mut data: Item) -> Result<Item> {
    validate_item(&data)?;
    data.id = Uuid::new_v4().to_string();
    db.execute(
        "INSERT INTO items(id,name,hsn,stock,data) VALUES(?,?,?,?,?)",
        params![data.id, data.name, data.hsn, data.stock, json(&data)?],
    )
    .map_err(db_error)?;
    Ok(data)
}

pub fn items(db: &Connection, query: &str) -> Result<Vec<Item>> {
    let query = query.trim();
    let mut statement = db.prepare(
        "SELECT data,stock FROM items WHERE (?1='' OR instr(lower(name),lower(?1))>0 OR instr(hsn,?1)>0) ORDER BY name COLLATE NOCASE LIMIT ?2",
    ).map_err(db_error)?;
    let records = statement
        .query_map(params![query, PAGE_SIZE], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<i64>>(1)?))
        })
        .map_err(db_error)?;
    records
        .map(|result| {
            let (data, stock) = result.map_err(db_error)?;
            let mut item: Item = parse(&data)?;
            item.stock = stock;
            Ok(item)
        })
        .collect()
}

pub fn customers(db: &Connection, query: &str) -> Result<Vec<Customer>> {
    let mut statement = db.prepare(
        "SELECT data FROM customers WHERE (?1='' OR instr(lower(name),lower(?1))>0 OR instr(json_extract(data,'$.gstin'),upper(?1))>0 OR instr(json_extract(data,'$.email'),lower(?1))>0 OR (?3<>'' AND instr(json_extract(data,'$.phone'),?3)>0)) ORDER BY name COLLATE NOCASE LIMIT ?2",
    ).map_err(db_error)?;
    let query = query.trim();
    let phone: String = if query.bytes().all(|byte| {
        byte.is_ascii_digit() || matches!(byte, b'+' | b' ' | b'-' | b'(' | b')' | b'.')
    }) {
        query
            .bytes()
            .filter(u8::is_ascii_digit)
            .map(char::from)
            .collect()
    } else {
        String::new()
    };
    let records = statement
        .query_map(params![query, PAGE_SIZE, phone], |row| {
            row.get::<_, String>(0)
        })
        .map_err(db_error)?;
    records
        .map(|record| parse(&record.map_err(db_error)?))
        .collect()
}

pub fn selected_customer(db: &Connection, id: &str) -> Result<Customer> {
    let data: String = db
        .query_row("SELECT data FROM customers WHERE id=?", [id], |row| {
            row.get(0)
        })
        .map_err(db_error)?;
    parse(&data)
}

pub fn selected_items(db: &Connection, ids: &[String]) -> Result<Vec<Item>> {
    if ids.len() > 100 {
        return Err("Choose up to 100 items.".into());
    }
    let mut statement = db
        .prepare("SELECT data,stock FROM items WHERE id=?")
        .map_err(db_error)?;
    ids.iter()
        .map(|id| {
            let (data, stock): (String, Option<i64>) = statement
                .query_row([id], |row| Ok((row.get(0)?, row.get(1)?)))
                .map_err(db_error)?;
            let mut item: Item = parse(&data)?;
            item.stock = stock;
            Ok(item)
        })
        .collect()
}

pub(crate) fn require_business(db: &Connection) -> Result<Business> {
    business(db)?.ok_or_else(|| "Set up a business first.".into())
}

fn assemble(db: &Connection, input: &DraftInput) -> Result<(Invoice, bool)> {
    validate_draft(input)?;
    let seller = require_business(db)?;
    let (customer, insert_new) = contacts::resolve_customer(db, input)?;
    let items = selected_items(
        db,
        &input
            .lines
            .iter()
            .map(|line| line.item_id.clone())
            .collect::<Vec<_>>(),
    )?;
    Ok((calculate(input, seller, customer, items)?, insert_new))
}

pub fn review(db: &Connection, input: &DraftInput) -> Result<Invoice> {
    assemble(db, input).map(|(invoice, _)| invoice)
}

pub fn save_draft(db: &mut Connection, mut input: DraftInput) -> Result<DraftInput> {
    validate_draft(&input)?;
    input.new_customer = input
        .new_customer
        .take()
        .map(normalize_customer)
        .transpose()?;
    let tx = db
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(db_error)?;
    require_business(&tx)?;
    if let Some(id) = &input.id {
        let changed = tx
            .execute(
                "UPDATE drafts SET data=?,updated=CURRENT_TIMESTAMP WHERE id=?",
                params![json(&input)?, id],
            )
            .map_err(db_error)?;
        if changed != 1 {
            return Err("This draft is no longer open. Check the invoice register.".into());
        }
    } else {
        let id = Uuid::new_v4().to_string();
        input.id = Some(id.clone());
        tx.execute(
            "INSERT INTO drafts(id,data) VALUES(?,?)",
            params![id, json(&input)?],
        )
        .map_err(db_error)?;
    }
    tx.commit().map_err(db_error)?;
    Ok(input)
}

pub fn draft(db: &Connection, id: &str) -> Result<DraftInput> {
    let data: String = db
        .query_row("SELECT data FROM drafts WHERE id=?", [id], |row| row.get(0))
        .optional()
        .map_err(db_error)?
        .ok_or_else(|| "This draft is no longer open.".to_owned())?;
    parse(&data)
}

pub fn invoice(db: &Connection, id: &str) -> Result<Option<Invoice>> {
    db.query_row("SELECT data FROM invoices WHERE id=?", [id], |row| {
        row.get::<_, String>(0)
    })
    .optional()
    .map_err(db_error)?
    .map(|data| parse(&data))
    .transpose()
}

fn customer_name(invoice: &Invoice) -> &str {
    invoice
        .customer
        .as_ref()
        .map_or("Walk-in customer", |customer| &customer.name)
}

pub fn issue(db: &mut Connection, id: &str, template: Option<InvoiceTemplate>) -> Result<Invoice> {
    let tx = db
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(db_error)?;
    if let Some(issued) = invoice(&tx, id)? {
        return Ok(issued);
    }
    let mut input = draft(&tx, id)?;
    if let Some(template) = template {
        input.template = Some(template);
    }
    let (mut document, insert_new) = assemble(&tx, &input)?;
    if let Some(reason) = &document.blocked_reason {
        return Err(reason.clone());
    }
    let year = fiscal_year(&input.date)?;
    let next = tx
        .query_row(
            "SELECT next FROM sequences WHERE fiscal_year=?",
            [&year],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(db_error)?
        .unwrap_or(1);
    let number = format!("{}/{year}/{next:04}", document.seller.prefix);
    if number.len() > 16 {
        return Err(
            "The invoice series has reached its maximum length for this financial year.".into(),
        );
    }
    if insert_new {
        if let Some(customer) = document.customer.as_mut() {
            contacts::insert_customer(&tx, customer)?;
        }
    }
    document.number = Some(number.clone());
    let tax = document.totals.cgst + document.totals.local_gst + document.totals.igst;
    tx.execute(
        "INSERT INTO invoices(id,number,date,customer,payment,total,tax,data) VALUES(?,?,?,?,?,?,?,?)",
        params![id, number, document.date, customer_name(&document), document.payment, document.totals.total, tax, json(&document)?],
    ).map_err(db_error)?;
    let mut quantities = BTreeMap::new();
    for line in &input.lines {
        *quantities.entry(&line.item_id).or_insert(0_i64) += line.quantity;
    }
    for (item_id, quantity) in quantities {
        let (name, stock): (String, Option<i64>) = tx
            .query_row(
                "SELECT name,stock FROM items WHERE id=?",
                [item_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(db_error)?;
        if let Some(stock) = stock {
            if stock < quantity {
                return Err(format!(
                    "Insufficient stock for {name}. The draft has been kept."
                ));
            }
            tx.execute(
                "UPDATE items SET stock=stock-? WHERE id=?",
                params![quantity, item_id],
            )
            .map_err(db_error)?;
            tx.execute(
                "INSERT INTO stock_movements(invoice_id,item_id,quantity) VALUES(?,?,?)",
                params![id, item_id, quantity],
            )
            .map_err(db_error)?;
        }
    }
    tx.execute("INSERT INTO sequences(fiscal_year,next) VALUES(?,?) ON CONFLICT(fiscal_year) DO UPDATE SET next=excluded.next", params![year, next + 1]).map_err(db_error)?;
    tx.execute("DELETE FROM drafts WHERE id=?", [id])
        .map_err(db_error)?;
    tx.execute(
        "INSERT INTO audit(action,reference) VALUES('invoice_issued',?)",
        [&number],
    )
    .map_err(db_error)?;
    tx.commit().map_err(db_error)?;
    Ok(document)
}

pub fn register(db: &Connection, status: &str, query: &str, page: u32) -> Result<Vec<RegisterRow>> {
    let offset = i64::from(page) * PAGE_SIZE;
    if status == "draft" {
        let mut statement = db
            .prepare("SELECT data FROM drafts ORDER BY updated DESC,id DESC LIMIT ? OFFSET ?")
            .map_err(db_error)?;
        let records = statement
            .query_map(params![PAGE_SIZE, offset], |row| row.get::<_, String>(0))
            .map_err(db_error)?;
        return records
            .map(|record| {
                let input: DraftInput = parse(&record.map_err(db_error)?)?;
                let customer = input
                    .customer_id
                    .as_ref()
                    .map(|id| {
                        db.query_row("SELECT name FROM customers WHERE id=?", [id], |row| {
                            row.get::<_, String>(0)
                        })
                        .optional()
                        .map_err(db_error)
                    })
                    .transpose()?
                    .flatten()
                    .or_else(|| input.new_customer.map(|customer| customer.name))
                    .unwrap_or_else(|| "Walk-in customer".into());
                Ok(RegisterRow {
                    id: input.id.unwrap_or_default(),
                    number: None,
                    date: input.date,
                    customer,
                    payment: input.payment,
                    totals: None,
                    currency: input.currency,
                })
            })
            .collect();
    }
    let mut statement = db.prepare("SELECT id,data FROM invoices WHERE (?1='' OR instr(lower(number),lower(?1))>0 OR instr(lower(customer),lower(?1))>0) ORDER BY date DESC,rowid DESC LIMIT ?2 OFFSET ?3").map_err(db_error)?;
    let records = statement
        .query_map(params![query.trim(), PAGE_SIZE, offset], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(db_error)?;
    records
        .map(|record| {
            let (id, json) = record.map_err(db_error)?;
            let document: Invoice = parse(&json)?;
            Ok(RegisterRow {
                id,
                customer: customer_name(&document).into(),
                number: document.number,
                date: document.date,
                payment: document.payment,
                totals: Some(document.totals),
                currency: document.currency,
            })
        })
        .collect()
}

pub fn dashboard(db: &Connection) -> Result<Dashboard> {
    db.query_row("SELECT count(*),coalesce(sum(total),0),coalesce(sum(CASE WHEN payment='Credit' THEN total ELSE 0 END),0),coalesce(sum(tax),0) FROM invoices WHERE coalesce(json_extract(data,'$.currency'),'INR') = 'INR'", [], |row| {
        Ok(Dashboard { invoice_count: row.get(0)?, sales: row.get(1)?, receivables: row.get(2)?, tax: row.get(3)? })
    }).map_err(db_error)
}

fn csv_cell(value: &str) -> String {
    let needs_prefix = value
        .trim_start_matches([' ', '\u{feff}'])
        .starts_with(['=', '+', '-', '@', '\t', '\n', '\r']);
    let prefix = if needs_prefix { "'" } else { "" };
    format!("\"{prefix}{}\"", value.replace('"', "\"\""))
}

fn csv_row(values: &[&str]) -> String {
    values
        .iter()
        .map(|value| csv_cell(value))
        .collect::<Vec<_>>()
        .join(",")
        + "\r\n"
}

fn rupees(paise: i64) -> String {
    format!("{}.{:02}", paise / 100, paise % 100)
}

pub fn export_csv(db: &Connection, month: &str, path: &Path, currency: Currency) -> Result<i64> {
    fiscal_year(&format!("{month}-01"))?;
    if path.extension().and_then(|value| value.to_str()) != Some("csv") {
        return Err("Choose a .csv file.".into());
    }
    let next_month = chrono::NaiveDate::parse_from_str(&format!("{month}-01"), "%Y-%m-%d")
        .map_err(db_error)?
        .checked_add_months(chrono::Months::new(1))
        .ok_or("Choose a valid month.")?
        .to_string();
    let mut stmt = db
        .prepare("SELECT data FROM invoices WHERE date>=? AND date<? AND coalesce(json_extract(data,'$.currency'),'INR')=? ORDER BY date ASC,rowid ASC")
        .map_err(db_error)?;
    let records = stmt
        .query_map(
            params![format!("{month}-01"), next_month, currency.code()],
            |row| row.get::<_, String>(0),
        )
        .map_err(db_error)?;
    let mut output = String::from("\u{feff}");
    let code = currency.code();
    output.push_str(&csv_row(&[
        "Invoice",
        "Date",
        "Customer",
        "GSTIN",
        "Place of supply",
        "Payment",
        &format!("Taxable {code}"),
        &format!("CGST {code}"),
        &format!("SGST {code}"),
        &format!("UTGST {code}"),
        &format!("IGST {code}"),
        &format!("Total {code}"),
    ]));
    let mut count = 0;
    for record in records {
        let bill: Invoice = parse(&record.map_err(db_error)?)?;
        let (sgst, utgst) = if bill.local_tax_name == "SGST" {
            (bill.totals.local_gst, 0)
        } else {
            (0, bill.totals.local_gst)
        };
        output.push_str(&csv_row(&[
            bill.number.as_deref().unwrap_or_default(),
            &bill.date,
            customer_name(&bill),
            bill.customer
                .as_ref()
                .map_or("", |customer| &customer.gstin),
            &bill.place,
            &bill.payment,
            &rupees(bill.totals.taxable),
            &rupees(bill.totals.cgst),
            &rupees(sgst),
            &rupees(utgst),
            &rupees(bill.totals.igst),
            &rupees(bill.totals.total),
        ]));
        count += 1;
    }
    write_new(path, output.as_bytes())?;
    Ok(count)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(db_error)?;
    if let Err(error) = file.write_all(bytes).and_then(|_| file.sync_all()) {
        drop(file);
        let _ = fs::remove_file(path);
        return Err(db_error(error));
    }
    Ok(())
}

fn copy_database(source: &Connection, target: &mut Connection) -> Result<()> {
    Backup::new(source, target)
        .map_err(db_error)?
        .run_to_completion(128, Duration::from_millis(15), None)
        .map_err(db_error)
}

pub fn snapshot(db: &Connection, path: &Path) -> Result<()> {
    if path.extension().and_then(|value| value.to_str()) != Some("ascelbill") {
        return Err("Choose an .ascelbill backup file.".into());
    }
    write_new(path, &[])?;
    let write = (|| {
        let mut target = Connection::open(path).map_err(db_error)?;
        copy_database(db, &mut target)?;
        verify_schema(&target)?;
        let integrity: String = target
            .query_row("PRAGMA quick_check(1)", [], |row| row.get(0))
            .map_err(db_error)?;
        if integrity != "ok" {
            return Err("The snapshot failed its integrity check.".into());
        }
        Ok(())
    })();
    if write.is_err() {
        let _ = fs::remove_file(path);
    }
    write
}

pub fn restore(db: &mut Connection, path: &Path, directory: &Path) -> Result<PathBuf> {
    if path.extension().and_then(|value| value.to_str()) != Some("ascelbill") {
        return Err("Choose an .ascelbill backup file.".into());
    }
    let mut source =
        Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(db_error)?;
    source
        .execute_batch("PRAGMA trusted_schema=OFF; PRAGMA query_only=ON;")
        .map_err(db_error)?;
    let source = source.transaction().map_err(db_error)?;
    verify_schema(&source)?;
    let integrity: String = source
        .query_row("PRAGMA quick_check(1)", [], |row| row.get(0))
        .map_err(db_error)?;
    if integrity != "ok" {
        return Err("That backup failed its integrity check.".into());
    }
    let tables: i64 = source.query_row("SELECT count(*) FROM sqlite_schema WHERE type='table' AND name IN ('business','customers','items','drafts','sequences','invoices','stock_movements','audit')", [], |row| row.get(0)).map_err(db_error)?;
    if tables != 8 || business(&source)?.is_none() {
        return Err("That backup does not contain a complete business workspace.".into());
    }
    {
        let mut current = db
            .prepare("SELECT id,data FROM invoices")
            .map_err(db_error)?;
        let mut incoming = source
            .prepare("SELECT data FROM invoices WHERE id=?")
            .map_err(db_error)?;
        let records = current
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(db_error)?;
        for record in records {
            let (id, document) = record.map_err(db_error)?;
            let restored: Option<String> = incoming
                .query_row([id], |row| row.get(0))
                .optional()
                .map_err(db_error)?;
            if restored.as_deref() != Some(document.as_str()) {
                return Err("This backup would replace or remove issued invoices. Choose a current backup or restore on an empty workstation.".into());
            }
        }
    }
    let recovery = directory.join(format!("before-restore-{}.ascelbill", Uuid::new_v4()));
    snapshot(db, &recovery)?;
    copy_database(&source, db)?;
    contacts::ensure_indexes(db)?;
    Ok(recovery)
}
