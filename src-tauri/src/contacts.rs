use crate::{
    model::*,
    store::{db_error, json, parse, require_business},
};
use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};
use serde::Serialize;
use std::collections::BTreeSet;
use uuid::Uuid;

pub fn ensure_indexes(db: &Connection) -> Result<()> {
    db.execute_batch(
        "CREATE UNIQUE INDEX IF NOT EXISTS customer_phone ON customers(json_extract(data,'$.phone')) WHERE json_extract(data,'$.phone') <> '';
         CREATE UNIQUE INDEX IF NOT EXISTS customer_email ON customers(json_extract(data,'$.email')) WHERE json_extract(data,'$.email') <> '';",
    ).map_err(db_error)
}

pub fn matched_customer(db: &Connection, phone: &str, email: &str) -> Result<Option<Customer>> {
    let phone = normalized_phone(phone)?;
    let email = normalized_email(email)?;
    if phone.is_empty() && email.is_empty() {
        return Ok(None);
    }
    let mut statement = db.prepare(
        "SELECT data FROM customers WHERE json_extract(data,'$.phone')<>'' AND json_extract(data,'$.phone')=?1
         UNION
         SELECT data FROM customers WHERE json_extract(data,'$.email')<>'' AND json_extract(data,'$.email')=?2",
    ).map_err(db_error)?;
    let mut matched = statement.query(params![phone, email]).map_err(db_error)?;
    let Some(record) = matched.next().map_err(db_error)? else {
        return Ok(None);
    };
    let data: String = record.get(0).map_err(db_error)?;
    if matched.next().map_err(db_error)?.is_some() {
        return Err("The phone and email belong to different customers.".into());
    }
    parse(&data).map(Some)
}

pub fn add_customer(db: &Connection, data: Customer) -> Result<Customer> {
    let tx = Transaction::new_unchecked(db, TransactionBehavior::Immediate).map_err(db_error)?;
    require_business(&tx)?;
    let mut data = normalize_customer(data)?;
    if let Some(existing) = matched_customer(&tx, &data.phone, &data.email)? {
        if existing.id != data.id {
            return Err(format!(
                "This phone or email is already saved for {}.",
                existing.name
            ));
        }
    }
    if data.id.is_empty() {
        insert_customer(&tx, &mut data)?;
    } else {
        let count = tx
            .execute(
                "UPDATE customers SET name=?,data=? WHERE id=?",
                params![data.name, json(&data)?, data.id],
            )
            .map_err(db_error)?;
        if count != 1 {
            return Err("That customer could not be found.".into());
        }
    }
    tx.commit().map_err(db_error)?;
    Ok(data)
}

pub fn insert_customer(db: &Connection, data: &mut Customer) -> Result<()> {
    data.id = Uuid::new_v4().to_string();
    db.execute(
        "INSERT INTO customers(id,name,data) VALUES(?,?,?)",
        params![data.id, data.name, json(data)?],
    )
    .map_err(db_error)?;
    Ok(())
}

pub fn resolve_customer(db: &Connection, input: &DraftInput) -> Result<(Option<Customer>, bool)> {
    if let Some(id) = &input.customer_id {
        let data: String = db
            .query_row("SELECT data FROM customers WHERE id=?", [id], |row| {
                row.get(0)
            })
            .optional()
            .map_err(db_error)?
            .ok_or("Choose an existing customer.")?;
        return parse(&data).map(|customer| (Some(customer), false));
    }
    let Some(new) = &input.new_customer else {
        return Ok((None, false));
    };
    let candidate = normalize_customer(new.clone())?;
    if let Some(existing) = matched_customer(db, &candidate.phone, &candidate.email)? {
        let details_conflict = candidate.name.to_lowercase() != existing.name.to_lowercase()
            || [
                (&candidate.gstin, &existing.gstin),
                (&candidate.phone, &existing.phone),
                (&candidate.email, &existing.email),
            ]
            .iter()
            .any(|(entered, saved)| !entered.is_empty() && entered != saved);
        if details_conflict {
            return Err(format!("This phone or email is saved for {}. Select the existing customer or correct the details.", existing.name));
        }
        return Ok((Some(existing), false));
    }
    Ok((Some(candidate), true))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSummary {
    pub ready: usize,
    pub imported: usize,
    pub duplicates: usize,
    pub invalid: usize,
    pub issues: Vec<ImportIssue>,
}

#[derive(Debug, Serialize)]
pub struct ImportIssue {
    line: usize,
    message: String,
}

pub fn import_csv(db: &mut Connection, source: &str, apply: bool) -> Result<ImportSummary> {
    if source.len() > 1_000_000 {
        return Err("Choose a UTF-8 CSV file up to 1 MB.".into());
    }
    let tx = db
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(db_error)?;
    let business = require_business(&tx)?;
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(source.trim_start_matches('\u{feff}').as_bytes());
    let header = reader
        .headers()
        .map_err(|_| "The CSV header could not be read.")?
        .clone();
    let column = |aliases: &[&str]| {
        header.iter().position(|field| {
            aliases
                .iter()
                .any(|alias| field.eq_ignore_ascii_case(alias))
        })
    };
    let name = column(&["name", "customer name"]).ok_or("Add a name column to the CSV.")?;
    let phone = column(&["phone", "mobile", "phone number"]);
    let email = column(&["email", "email address"]);
    if phone.is_none() && email.is_none() {
        return Err("Add a phone or email column to the CSV.".into());
    }
    let address = column(&["address", "billing address"]);
    let state = column(&["state", "state code", "state_code"]);
    let gstin = column(&["gstin"]);
    let mut summary = ImportSummary {
        ready: 0,
        imported: 0,
        duplicates: 0,
        invalid: 0,
        issues: Vec::new(),
    };
    let mut seen_phone = BTreeSet::new();
    let mut seen_email = BTreeSet::new();
    for (index, entry) in reader.records().enumerate() {
        if index >= 3000 {
            return Err("Import up to 3,000 customer rows at a time.".into());
        }
        let line = entry
            .as_ref()
            .ok()
            .and_then(csv::StringRecord::position)
            .or_else(|| entry.as_ref().err().and_then(csv::Error::position))
            .map_or(index + 2, |position| position.line() as usize);
        let parsed = entry
            .map_err(|_| "Check this row's columns and UTF-8 encoding.".to_owned())
            .and_then(|row| {
                let value = |index: Option<usize>| {
                    index.and_then(|index| row.get(index)).unwrap_or_default()
                };
                let tax_id = value(gstin);
                let state = csv_state(value(state), tax_id, &business.state)?;
                let data = normalize_customer(Customer {
                    id: String::new(),
                    name: value(Some(name)).to_owned(),
                    phone: value(phone).to_owned(),
                    email: value(email).to_owned(),
                    state,
                    gstin: tax_id.to_owned(),
                    address: value(address).to_owned(),
                })?;
                if data.phone.is_empty() && data.email.is_empty() {
                    return Err("Add a phone or email.".into());
                }
                Ok(data)
            });
        let message = match parsed {
            Ok(mut data) => {
                let repeated = (!data.phone.is_empty() && seen_phone.contains(&data.phone))
                    || (!data.email.is_empty() && seen_email.contains(&data.email));
                match matched_customer(&tx, &data.phone, &data.email) {
                    Err(reason) => {
                        summary.invalid += 1;
                        reason
                    }
                    Ok(existing) if repeated || existing.is_some() => {
                        summary.duplicates += 1;
                        "Phone or email already appears in the workspace or file.".into()
                    }
                    Ok(_) => {
                        if !data.phone.is_empty() {
                            seen_phone.insert(data.phone.clone());
                        }
                        if !data.email.is_empty() {
                            seen_email.insert(data.email.clone());
                        }
                        summary.ready += 1;
                        if apply {
                            insert_customer(&tx, &mut data)?;
                            summary.imported += 1;
                        }
                        continue;
                    }
                }
            }
            Err(reason) => {
                summary.invalid += 1;
                reason
            }
        };
        if summary.issues.len() < 25 {
            summary.issues.push(ImportIssue { line, message });
        }
    }
    if summary.ready + summary.duplicates + summary.invalid == 0 {
        return Err("The file has headers but no customer rows.".into());
    }
    if apply {
        tx.commit().map_err(db_error)?;
    }
    Ok(summary)
}

fn csv_state(value: &str, gstin: &str, fallback: &str) -> Result<String> {
    let value = if value.is_empty() {
        gstin.get(..2).unwrap_or(fallback)
    } else {
        value
    };
    let numeric = value
        .parse::<u8>()
        .ok()
        .map(|number| format!("{number:02}"));
    STATES
        .iter()
        .find(|(code, name)| {
            *code == value || name.eq_ignore_ascii_case(value) || numeric.as_deref() == Some(*code)
        })
        .map(|(code, _)| (*code).into())
        .ok_or_else(|| "Use a valid state name or GST state code.".into())
}
