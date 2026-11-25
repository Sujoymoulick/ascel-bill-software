use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    #[default]
    Inr,
    Usd,
}

impl Currency {
    pub fn code(self) -> &'static str {
        match self {
            Self::Inr => "INR",
            Self::Usd => "USD",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StockMode {
    Receive,
    Count,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InvoiceTemplate {
    #[default]
    Standard,
    Ledger,
    LaserA5,
    LaserA6,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Accent {
    #[default]
    Green,
    Navy,
    Black,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PrintSettings {
    pub template: InvoiceTemplate,
    pub accent: Accent,
    pub logo: String,
    pub footer: String,
    pub show_contacts: bool,
    pub show_payment_reference: bool,
}

impl Default for PrintSettings {
    fn default() -> Self {
        Self {
            template: InvoiceTemplate::Standard,
            accent: Accent::Green,
            logo: String::new(),
            footer: String::new(),
            show_contacts: false,
            show_payment_reference: true,
        }
    }
}

impl PrintSettings {
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Business {
    pub name: String,
    pub address: String,
    pub state: String,
    pub gstin: String,
    pub prefix: String,
    pub irn_required: bool,
    #[serde(default)]
    pub allow_usd: bool,
    #[serde(default, skip_serializing_if = "PrintSettings::is_default")]
    pub print_settings: PrintSettings,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Customer {
    pub id: String,
    pub name: String,
    pub address: String,
    pub state: String,
    pub gstin: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub phone: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub email: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: String,
    pub name: String,
    pub hsn: String,
    pub unit: String,
    pub price: i64,
    #[serde(default)]
    pub usd_price: Option<i64>,
    pub rate: i64,
    pub stock: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LineInput {
    pub item_id: String,
    pub quantity: i64,
    pub price: i64,
    pub discount: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftInput {
    pub id: Option<String>,
    pub date: String,
    pub customer_id: Option<String>,
    #[serde(default)]
    pub new_customer: Option<Customer>,
    #[serde(default)]
    pub template: Option<InvoiceTemplate>,
    #[serde(default)]
    pub currency: Currency,
    pub place: String,
    pub delivery_address: String,
    pub payment: String,
    pub reference: String,
    pub lines: Vec<LineInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Line {
    pub name: String,
    pub hsn: String,
    pub unit: String,
    pub quantity: i64,
    pub price: i64,
    pub discount: i64,
    pub rate: i64,
    pub taxable: i64,
    pub cgst: i64,
    pub local_gst: i64,
    pub igst: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Totals {
    pub taxable: i64,
    pub discount: i64,
    pub cgst: i64,
    pub local_gst: i64,
    pub igst: i64,
    pub total: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Invoice {
    pub number: Option<String>,
    #[serde(default)]
    pub template: Option<InvoiceTemplate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub print_settings: Option<PrintSettings>,
    #[serde(default)]
    pub currency: Currency,
    pub date: String,
    pub seller: Business,
    pub customer: Option<Customer>,
    pub place: String,
    pub delivery_address: String,
    pub payment: String,
    pub reference: String,
    pub local_tax_name: String,
    pub lines: Vec<Line>,
    pub totals: Totals,
    pub blocked_reason: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRow {
    pub id: String,
    pub number: Option<String>,
    pub date: String,
    pub customer: String,
    pub payment: String,
    pub totals: Option<Totals>,
    pub currency: Currency,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dashboard {
    pub invoice_count: i64,
    pub sales: i64,
    pub receivables: i64,
    pub tax: i64,
}

pub const REORDER_THRESHOLD: i64 = 5_000;

pub const STATES: &[(&str, &str)] = &[
    ("01", "Jammu and Kashmir"),
    ("02", "Himachal Pradesh"),
    ("03", "Punjab"),
    ("04", "Chandigarh"),
    ("05", "Uttarakhand"),
    ("06", "Haryana"),
    ("07", "Delhi"),
    ("08", "Rajasthan"),
    ("09", "Uttar Pradesh"),
    ("10", "Bihar"),
    ("11", "Sikkim"),
    ("12", "Arunachal Pradesh"),
    ("13", "Nagaland"),
    ("14", "Manipur"),
    ("15", "Mizoram"),
    ("16", "Tripura"),
    ("17", "Meghalaya"),
    ("18", "Assam"),
    ("19", "West Bengal"),
    ("20", "Jharkhand"),
    ("21", "Odisha"),
    ("22", "Chhattisgarh"),
    ("23", "Madhya Pradesh"),
    ("24", "Gujarat"),
    ("26", "Dadra and Nagar Haveli and Daman and Diu"),
    ("27", "Maharashtra"),
    ("29", "Karnataka"),
    ("30", "Goa"),
    ("31", "Lakshadweep"),
    ("32", "Kerala"),
    ("33", "Tamil Nadu"),
    ("34", "Puducherry"),
    ("35", "Andaman and Nicobar Islands"),
    ("36", "Telangana"),
    ("37", "Andhra Pradesh"),
    ("38", "Ladakh"),
];

fn state_exists(state: &str) -> bool {
    STATES.iter().any(|entry| entry.0 == state)
}

pub fn text(value: &str, label: &str, max: usize, required: bool) -> Result<()> {
    let length = value.trim().chars().count();
    if length > max || (required && length == 0) || value.contains('\0') {
        return Err(format!("Check {label}."));
    }
    Ok(())
}

fn gstin(value: &str, state: &str) -> Result<()> {
    if !state_exists(state) {
        return Err("Choose a valid state or union territory.".into());
    }
    if value.is_empty() {
        return Ok(());
    }
    let bytes = value.as_bytes();
    if bytes.len() != 15
        || !value.starts_with(state)
        || !bytes[2..7].iter().all(u8::is_ascii_uppercase)
        || !bytes[7..11].iter().all(u8::is_ascii_digit)
        || !bytes[11].is_ascii_uppercase()
        || !bytes[12].is_ascii_alphanumeric()
        || bytes[13] != b'Z'
        || !bytes[14].is_ascii_alphanumeric()
    {
        return Err("Enter a 15-character GSTIN matching the selected state.".into());
    }
    Ok(())
}

pub fn validate_business(business: &Business) -> Result<()> {
    text(&business.name, "the business name", 100, true)?;
    text(&business.address, "the business address", 350, true)?;
    gstin(&business.gstin, &business.state)?;
    if business.prefix.is_empty()
        || business.prefix.len() > 5
        || !business
            .prefix
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
    {
        return Err("Use 1 to 5 letters or digits for the invoice prefix.".into());
    }
    if business.irn_required && business.gstin.is_empty() {
        return Err("E-invoicing requires a GST-registered business.".into());
    }
    if business.allow_usd && !business.gstin.is_empty() {
        return Err("USD is available for ordinary invoices without GST.".into());
    }
    validate_print_settings(&business.print_settings)
}

pub fn validate_print_settings(settings: &PrintSettings) -> Result<()> {
    text(&settings.footer, "the invoice footer", 280, false)?;
    if settings.logo.is_empty() {
        return Ok(());
    }
    let encoded = settings
        .logo
        .strip_prefix("data:image/png;base64,")
        .or_else(|| settings.logo.strip_prefix("data:image/jpeg;base64,"));
    if !encoded.is_some_and(|bytes| {
        !bytes.is_empty()
            && bytes.len() <= 349_528
            && bytes
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'='))
    }) {
        return Err("Choose a PNG or JPEG logo up to 256 KB.".into());
    }
    Ok(())
}

pub fn validate_customer(customer: &Customer) -> Result<()> {
    text(&customer.name, "the customer name", 100, true)?;
    text(
        &customer.address,
        "the customer address",
        350,
        !customer.gstin.is_empty(),
    )?;
    gstin(&customer.gstin, &customer.state)
}

pub fn normalized_phone(value: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(String::new());
    }
    let digits: String = value
        .bytes()
        .filter(u8::is_ascii_digit)
        .map(char::from)
        .collect();
    if value.len() <= 40
        && value
            .trim_start_matches('+')
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b' ' | b'-' | b'(' | b')' | b'.'))
        && !value.starts_with("++")
    {
        match digits.len() {
            10 if !value.starts_with('+') => return Ok(format!("+91{digits}")),
            11 if !value.starts_with('+') && digits.starts_with('0') => {
                return Ok(format!("+91{}", &digits[1..]))
            }
            12 if digits.starts_with("91") => return Ok(format!("+{digits}")),
            8..=15 if value.starts_with('+') && !digits.starts_with('0') => {
                return Ok(format!("+{digits}"))
            }
            _ => {}
        }
    }
    Err("Enter a 10-digit Indian phone number or include the international country code.".into())
}

pub fn normalized_email(value: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(String::new());
    }
    if value.len() <= 254
        && !value
            .chars()
            .any(|ch| ch.is_whitespace() || ch.is_control())
    {
        if let Some((local, host)) = value.split_once('@') {
            if !local.is_empty()
                && host.contains('.')
                && !host.starts_with('.')
                && !host.ends_with('.')
                && !host.contains('@')
            {
                return Ok(value.to_ascii_lowercase());
            }
        }
    }
    Err("Enter a valid email address.".into())
}

pub fn normalize_customer(mut customer: Customer) -> Result<Customer> {
    customer.name = customer.name.trim().to_owned();
    customer.address = customer.address.trim().to_owned();
    customer.gstin = customer.gstin.trim().to_ascii_uppercase();
    customer.phone = normalized_phone(&customer.phone)?;
    customer.email = normalized_email(&customer.email)?;
    validate_customer(&customer)?;
    Ok(customer)
}

pub fn validate_item(item: &Item) -> Result<()> {
    text(&item.name, "the item name", 100, true)?;
    text(&item.unit, "the unit", 12, true)?;
    if !matches!(item.hsn.len(), 0 | 4 | 6 | 8)
        || !item.hsn.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err("HSN must contain 4, 6 or 8 digits when supplied.".into());
    }
    if !(0..=100_000_000).contains(&item.price)
        || !(0..=10_000).contains(&item.rate)
        || item
            .usd_price
            .is_some_and(|price| !(0..=100_000_000).contains(&price))
    {
        return Err("Enter a valid selling price and GST rate.".into());
    }
    if item
        .stock
        .is_some_and(|value| !(0..=1_000_000_000).contains(&value))
    {
        return Err("Check the opening stock quantity.".into());
    }
    Ok(())
}

pub fn fiscal_year(date: &str) -> Result<String> {
    let parsed = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|_| "Enter a valid invoice date.".to_owned())?;
    if !(2017..=2099).contains(&parsed.year()) || parsed.to_string() != date {
        return Err("Enter a date in YYYY-MM-DD format from 2017 through 2099.".into());
    }
    let start = parsed.year() - i32::from(parsed.month() < 4);
    Ok(format!("{:02}-{:02}", start % 100, (start + 1) % 100))
}

pub fn validate_draft(input: &DraftInput) -> Result<()> {
    fiscal_year(&input.date)?;
    if input.customer_id.is_some() && input.new_customer.is_some() {
        return Err("Choose one customer for this invoice.".into());
    }
    if !state_exists(&input.place)
        || !matches!(
            input.payment.as_str(),
            "Credit" | "Cash" | "UPI" | "Bank transfer" | "Card"
        )
    {
        return Err("Check place of supply and payment method.".into());
    }
    text(&input.delivery_address, "the delivery address", 350, false)?;
    text(&input.reference, "the payment reference", 100, false)?;
    if input.lines.len() > 100 {
        return Err("An invoice supports up to 100 items.".into());
    }
    for line in &input.lines {
        if !(1..=10_000_000).contains(&line.quantity)
            || !(0..=100_000_000).contains(&line.price)
            || !(0..=1_000_000_000_000).contains(&line.discount)
        {
            return Err("Check item quantity, price and discount.".into());
        }
    }
    Ok(())
}

fn round(value: i64, divisor: i64) -> i64 {
    (value + divisor / 2) / divisor
}

pub fn calculate(
    input: &DraftInput,
    mut seller: Business,
    customer: Option<Customer>,
    items: Vec<Item>,
) -> Result<Invoice> {
    validate_draft(input)?;
    if input.lines.is_empty() {
        return Err("Add an item before reviewing the invoice.".into());
    }
    let registered = !seller.gstin.is_empty();
    if input.currency == Currency::Usd && (registered || !seller.allow_usd) {
        return Err("USD is available for ordinary invoices without GST.".into());
    }
    let intra = seller.state == input.place;
    let local_tax_name = if matches!(seller.state.as_str(), "04" | "26" | "31" | "35" | "38") {
        "UTGST"
    } else {
        "SGST"
    };
    let mut totals = Totals::default();
    let mut lines = Vec::with_capacity(items.len());
    for (entry, item) in input.lines.iter().zip(items) {
        if registered && item.hsn.is_empty() {
            return Err(format!(
                "Add an HSN for {} before issuing a GST invoice.",
                item.name
            ));
        }
        let base = round(entry.quantity * entry.price, 1_000);
        if entry.discount > base {
            return Err(format!(
                "The discount on {} exceeds its line amount.",
                item.name
            ));
        }
        let taxable = base - entry.discount;
        let rate = if registered { item.rate } else { 0 };
        let component = if intra {
            round(taxable * rate, 20_000)
        } else {
            0
        };
        let igst = if intra {
            0
        } else {
            round(taxable * rate, 10_000)
        };
        totals.taxable += taxable;
        totals.discount += entry.discount;
        totals.cgst += component;
        totals.local_gst += component;
        totals.igst += igst;
        lines.push(Line {
            name: item.name,
            hsn: item.hsn,
            unit: item.unit,
            quantity: entry.quantity,
            price: entry.price,
            discount: entry.discount,
            rate,
            taxable,
            cgst: component,
            local_gst: component,
            igst,
        });
    }
    totals.total = totals.taxable + totals.cgst + totals.local_gst + totals.igst;
    let b2b = customer
        .as_ref()
        .is_some_and(|party| !party.gstin.is_empty());
    let blocked_reason = if registered && seller.irn_required && b2b {
        Some("This invoice needs an IRN and signed QR. Keep it as a draft and issue it through your approved e-invoicing workflow.".into())
    } else if registered
        && !b2b
        && totals.taxable >= 5_000_000
        && customer
            .as_ref()
            .is_none_or(|party| party.address.trim().is_empty())
    {
        Some("Enter the customer's name and address for this sale.".into())
    } else {
        None
    };
    let mut print_settings = std::mem::take(&mut seller.print_settings);
    let template = input.template.unwrap_or(print_settings.template);
    print_settings.template = template;
    seller.allow_usd = false;
    Ok(Invoice {
        number: None,
        template: Some(template),
        print_settings: Some(print_settings),
        currency: input.currency,
        date: input.date.clone(),
        seller,
        customer,
        place: input.place.clone(),
        delivery_address: input.delivery_address.clone(),
        payment: input.payment.clone(),
        reference: input.reference.clone(),
        local_tax_name: local_tax_name.into(),
        lines,
        totals,
        blocked_reason,
    })
}
