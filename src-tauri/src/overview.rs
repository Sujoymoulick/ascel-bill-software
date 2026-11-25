use crate::{model::Result, store::db_error};
use chrono::{Days, NaiveDate};
use rusqlite::{params, Connection};
use serde::Serialize;

const PERIOD_DAYS: u64 = 30;
const REORDER_THRESHOLD: i64 = 5_000;
const STOCK_ALERT_LIMIT: i64 = 6;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    period_days: u64,
    start_date: String,
    end_date: String,
    sales: Sales,
    customer_count: i64,
    active_customer_count: i64,
    item_count: i64,
    tracked_item_count: i64,
    in_stock_count: i64,
    low_stock_count: i64,
    out_of_stock_count: i64,
    untracked_item_count: i64,
    reorder_threshold: i64,
    stock_alerts: Vec<StockAlert>,
}

#[derive(Debug, Serialize)]
struct Sales {
    #[serde(rename = "INR")]
    inr: CurrencySales,
    #[serde(rename = "USD")]
    usd: CurrencySales,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CurrencySales {
    invoice_count: i64,
    total: i64,
    credit_sales: i64,
    tax: i64,
    days: Vec<DailySales>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct DailySales {
    date: String,
    invoice_count: i64,
    total: i64,
    credit_sales: i64,
    tax: i64,
}

#[derive(Debug, Serialize)]
struct StockAlert {
    id: String,
    name: String,
    unit: String,
    stock: i64,
}

impl CurrencySales {
    fn empty(start: NaiveDate) -> Self {
        let days = start
            .iter_days()
            .take(PERIOD_DAYS as usize)
            .map(|day| DailySales {
                date: day.to_string(),
                ..DailySales::default()
            })
            .collect();
        Self {
            invoice_count: 0,
            total: 0,
            credit_sales: 0,
            tax: 0,
            days,
        }
    }

    fn record(&mut self, start: NaiveDate, day: DailySales) -> Result<()> {
        let date = day.date.parse::<NaiveDate>().map_err(db_error)?;
        let index = date.signed_duration_since(start).num_days() as usize;
        let slot = self
            .days
            .get_mut(index)
            .ok_or("Invalid sales date in the local overview.")?;
        self.invoice_count += day.invoice_count;
        self.total += day.total;
        self.credit_sales += day.credit_sales;
        self.tax += day.tax;
        *slot = day;
        Ok(())
    }
}

pub fn load(db: &Connection, as_of: &str) -> Result<Overview> {
    let end = as_of
        .parse::<NaiveDate>()
        .ok()
        .filter(|date| as_of.len() == 10 && date.to_string() == as_of)
        .ok_or("Choose a valid overview date in YYYY-MM-DD format.")?;
    let start = end
        .checked_sub_days(Days::new(PERIOD_DAYS - 1))
        .ok_or("Choose a valid overview date in YYYY-MM-DD format.")?;
    let start_date = start.to_string();
    let (customer_count, active_customer_count) = db
        .query_row(
            "SELECT
           (SELECT COUNT(*) FROM customers),
           (SELECT COUNT(DISTINCT NULLIF(json_extract(data,'$.customer.id'), ''))
            FROM invoices WHERE date BETWEEN ?1 AND ?2)",
            params![start_date, as_of],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(db_error)?;

    let (item_count, tracked_item_count, in_stock_count, low_stock_count, out_of_stock_count) = db
        .query_row(
            "SELECT COUNT(*), COUNT(stock),
                COUNT(*) FILTER (WHERE stock > ?1),
                COUNT(*) FILTER (WHERE stock BETWEEN 1 AND ?1),
                COUNT(*) FILTER (WHERE stock = 0)
         FROM items",
            [REORDER_THRESHOLD],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .map_err(db_error)?;

    Ok(Overview {
        period_days: PERIOD_DAYS,
        sales: load_sales(db, start, &start_date, as_of)?,
        stock_alerts: load_stock_alerts(db)?,
        start_date,
        end_date: as_of.to_owned(),
        customer_count,
        active_customer_count,
        item_count,
        tracked_item_count,
        in_stock_count,
        low_stock_count,
        out_of_stock_count,
        untracked_item_count: item_count - tracked_item_count,
        reorder_threshold: REORDER_THRESHOLD,
    })
}

fn load_sales(db: &Connection, start: NaiveDate, start_date: &str, as_of: &str) -> Result<Sales> {
    let mut sales = Sales {
        inr: CurrencySales::empty(start),
        usd: CurrencySales::empty(start),
    };
    let mut statement = db
        .prepare(
            "SELECT date,
                COALESCE(json_extract(data, '$.currency'), 'INR') AS currency,
                COUNT(*), SUM(total),
                SUM(CASE WHEN payment = 'Credit' THEN total ELSE 0 END),
                SUM(tax)
         FROM invoices WHERE date BETWEEN ?1 AND ?2
         GROUP BY date, currency",
        )
        .map_err(db_error)?;
    let rows = statement
        .query_map(params![start_date, as_of], |row| {
            let currency: String = row.get(1)?;
            let daily = DailySales {
                date: row.get(0)?,
                invoice_count: row.get(2)?,
                total: row.get(3)?,
                credit_sales: row.get(4)?,
                tax: row.get(5)?,
            };
            Ok((currency, daily))
        })
        .map_err(db_error)?;
    for row in rows {
        let (currency, day) = row.map_err(db_error)?;
        let series = match currency.as_str() {
            "INR" => &mut sales.inr,
            "USD" => &mut sales.usd,
            _ => return Err("Unsupported invoice currency in the local overview.".into()),
        };
        series.record(start, day)?;
    }
    Ok(sales)
}

fn load_stock_alerts(db: &Connection) -> Result<Vec<StockAlert>> {
    let mut statement = db
        .prepare(
            "SELECT id, name, COALESCE(json_extract(data, '$.unit'), ''), stock
         FROM items WHERE stock <= ?1
         ORDER BY stock, name COLLATE NOCASE, id LIMIT ?2",
        )
        .map_err(db_error)?;
    let rows = statement
        .query_map(params![REORDER_THRESHOLD, STOCK_ALERT_LIMIT], |row| {
            Ok(StockAlert {
                id: row.get(0)?,
                name: row.get(1)?,
                unit: row.get(2)?,
                stock: row.get(3)?,
            })
        })
        .map_err(db_error)?;
    rows.map(|row| row.map_err(db_error)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> Connection {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch(include_str!("schema.sql")).unwrap();
        db
    }

    #[test]
    fn fills_missing_days_and_keeps_currency_amounts_separate() {
        let db = db();
        db.execute_batch(
            r#"INSERT INTO customers VALUES ('c1','Local Shop','{}'), ('c2','No orders','{}');
             INSERT INTO invoices VALUES
             ('a','a','2024-02-15','Shop','Credit',10000,1800,'{"customer":{"id":"c1"}}'),
             ('b','b','2024-03-15','Shop','Cash',5000,0,'{"currency":"USD","customer":{"id":"c1"}}'),
             ('c','c','2024-03-15','Walk-in','Cash',2000,100,'{"currency":"INR"}'),
             ('d','d','2024-03-15','Walk-in','Cash',1000,50,'{"currency":null}'),
             ('e','e','2024-02-14','Walk-in','Credit',900,100,'{}'),
             ('f','f','2024-03-16','Walk-in','Credit',800,100,'{}');"#,
        ).unwrap();
        let overview = load(&db, "2024-03-15").unwrap();
        assert_eq!(
            (overview.start_date.as_str(), overview.end_date.as_str()),
            ("2024-02-15", "2024-03-15")
        );
        assert_eq!(
            (overview.customer_count, overview.active_customer_count),
            (2, 1)
        );
        assert_eq!(overview.sales.inr.days.len(), 30);
        assert_eq!(overview.sales.inr.days[14].date, "2024-02-29");
        assert_eq!(
            (
                overview.sales.inr.invoice_count,
                overview.sales.inr.total,
                overview.sales.inr.credit_sales,
                overview.sales.inr.tax
            ),
            (3, 13000, 10000, 1950)
        );
        assert_eq!(
            (
                overview.sales.usd.invoice_count,
                overview.sales.usd.total,
                overview.sales.usd.credit_sales
            ),
            (1, 5000, 0)
        );
        assert_eq!(overview.sales.inr.days[14].total, 0);
        assert_eq!(
            (
                overview.sales.inr.days[29].total,
                overview.sales.inr.days[29].invoice_count
            ),
            (3000, 2)
        );
        assert_eq!(overview.sales.usd.days[0].total, 0);
        let payload = serde_json::to_value(overview).unwrap();
        assert_eq!(payload["sales"]["INR"]["creditSales"], 10000);
        assert_eq!(payload["sales"]["USD"]["days"][29]["invoiceCount"], 1);
    }

    #[test]
    fn stock_groups_are_disjoint_and_critical_items_are_sorted_first() {
        let db = db();
        db.execute_batch(
            r#"INSERT INTO items VALUES
             ('healthy','Healthy','',5001,'{"unit":"kg"}'),
             ('exact','Exact','',5000,'{"unit":"box"}'),
             ('fraction','Fraction','',50,'{"unit":"kg"}'),
             ('none','None','',0,'{"unit":"pcs"}'),
             ('untracked','Untracked','',NULL,'{"unit":"pcs"}');"#,
        )
        .unwrap();
        let overview = load(&db, "2024-03-15").unwrap();
        assert_eq!(
            (
                overview.item_count,
                overview.tracked_item_count,
                overview.untracked_item_count
            ),
            (5, 4, 1)
        );
        assert_eq!(
            (
                overview.in_stock_count,
                overview.low_stock_count,
                overview.out_of_stock_count
            ),
            (1, 2, 1)
        );
        assert_eq!(overview.reorder_threshold, 5000);
        assert_eq!(overview.stock_alerts.len(), 3);
        assert_eq!(
            (
                overview.stock_alerts[0].id.as_str(),
                overview.stock_alerts[0].unit.as_str(),
                overview.stock_alerts[0].stock
            ),
            ("none", "pcs", 0)
        );
        assert_eq!(overview.stock_alerts[1].id, "fraction");
        assert_eq!(overview.stock_alerts[2].id, "exact");
        assert_eq!(
            overview.sales.inr.invoice_count + overview.sales.usd.invoice_count,
            0
        );
    }

    #[test]
    fn rejects_bad_dates() {
        let db = db();
        assert!(load(&db, "2024-02-30").is_err());
        assert!(load(&db, "2024-3-15").is_err());
        assert!(load(&db, "2024-03-15T12:00").is_err());
    }
}
