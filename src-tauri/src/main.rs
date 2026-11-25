#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod contacts;
mod model;
mod overview;
mod print;
mod store;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod workspace_tests;

use model::*;
use rusqlite::Connection;
use std::{path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Emitter, Manager, State};

struct Workstation {
    db: Mutex<Connection>,
    directory: PathBuf,
}

impl Workstation {
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.db
            .lock()
            .map_err(|_| "Restart Ascel Bill before accessing local data.".into())
    }
}

#[tauri::command]
fn workspace(state: State<'_, Workstation>) -> Result<Option<Business>> {
    store::business(&*state.lock()?)
}

#[tauri::command]
fn state_options() -> &'static [(&'static str, &'static str)] {
    STATES
}

#[tauri::command]
fn selected_items(state: State<'_, Workstation>, ids: Vec<String>) -> Result<Vec<Item>> {
    store::selected_items(&*state.lock()?, &ids)
}

#[tauri::command]
fn selected_customer(state: State<'_, Workstation>, id: &str) -> Result<Customer> {
    store::selected_customer(&*state.lock()?, id)
}

#[tauri::command]
fn save_business(state: State<'_, Workstation>, business: Business) -> Result<Business> {
    store::setup(&mut *state.lock()?, business)
}

#[tauri::command]
fn update_business(
    state: State<'_, Workstation>,
    name: String,
    address: String,
    irn_required: bool,
) -> Result<Business> {
    store::update_business(&mut *state.lock()?, name, address, irn_required)
}

#[tauri::command]
fn save_print_settings(state: State<'_, Workstation>, settings: PrintSettings) -> Result<Business> {
    store::save_print_settings(&mut *state.lock()?, settings)
}

#[tauri::command]
fn set_usd_option(state: State<'_, Workstation>, allowed: bool) -> Result<Business> {
    store::set_usd_option(&mut *state.lock()?, allowed)
}

fn notify_workspace(app: &AppHandle) {
    let _ = app.emit("workspace-changed", ());
}

#[tauri::command]
fn find_customer_contact(
    state: State<'_, Workstation>,
    phone: &str,
    email: &str,
) -> Result<Option<Customer>> {
    contacts::matched_customer(&*state.lock()?, phone, email)
}

#[tauri::command]
async fn import_customer_csv(
    app: AppHandle,
    content: String,
    apply: bool,
) -> Result<contacts::ImportSummary> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<Workstation>();
        let mut db = state.lock()?;
        let result = contacts::import_csv(&mut db, &content, apply)?;
        drop(db);
        if result.imported != 0 {
            notify_workspace(&app);
        }
        Ok(result)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
fn list_items(state: State<'_, Workstation>, query: &str) -> Result<Vec<Item>> {
    store::items(&*state.lock()?, query)
}

#[tauri::command]
fn list_customers(state: State<'_, Workstation>, query: &str) -> Result<Vec<Customer>> {
    store::customers(&*state.lock()?, query)
}

#[tauri::command]
fn save_item(app: AppHandle, state: State<'_, Workstation>, item: Item) -> Result<Item> {
    let saved = store::add_item(&*state.lock()?, item)?;
    notify_workspace(&app);
    Ok(saved)
}

#[tauri::command]
fn update_stock(
    app: AppHandle,
    state: State<'_, Workstation>,
    id: &str,
    mode: StockMode,
    quantity: i64,
    note: &str,
) -> Result<Item> {
    let saved = store::update_stock(&mut *state.lock()?, id, mode, quantity, note)?;
    notify_workspace(&app);
    Ok(saved)
}

#[tauri::command]
fn save_customer(
    app: AppHandle,
    state: State<'_, Workstation>,
    customer: Customer,
) -> Result<Customer> {
    let saved = store::add_customer(&*state.lock()?, customer)?;
    notify_workspace(&app);
    Ok(saved)
}

#[tauri::command]
fn save_draft(state: State<'_, Workstation>, input: DraftInput) -> Result<DraftInput> {
    store::save_draft(&mut *state.lock()?, input)
}

#[tauri::command]
fn get_draft(state: State<'_, Workstation>, id: &str) -> Result<DraftInput> {
    store::draft(&*state.lock()?, id)
}

#[tauri::command]
fn review_invoice(state: State<'_, Workstation>, input: DraftInput) -> Result<Invoice> {
    store::review(&*state.lock()?, &input)
}

#[tauri::command]
fn issue_invoice(
    app: AppHandle,
    state: State<'_, Workstation>,
    id: &str,
    template: Option<InvoiceTemplate>,
) -> Result<Invoice> {
    let invoice = store::issue(&mut *state.lock()?, id, template)?;
    notify_workspace(&app);
    Ok(invoice)
}

#[tauri::command]
fn get_invoice(state: State<'_, Workstation>, id: &str) -> Result<Invoice> {
    store::invoice(&*state.lock()?, id)?.ok_or_else(|| "That invoice could not be found.".into())
}

#[tauri::command]
fn invoice_register(
    state: State<'_, Workstation>,
    status: &str,
    query: &str,
    page: u32,
) -> Result<Vec<RegisterRow>> {
    store::register(&*state.lock()?, status, query, page)
}

#[tauri::command]
fn dashboard(state: State<'_, Workstation>) -> Result<Dashboard> {
    store::dashboard(&*state.lock()?)
}

#[tauri::command]
fn overview(state: State<'_, Workstation>, as_of: &str) -> Result<overview::Overview> {
    overview::load(&*state.lock()?, as_of)
}

#[tauri::command]
fn print_invoice(window: tauri::WebviewWindow, template: InvoiceTemplate) -> Result<()> {
    print::invoice(window, template)
}

#[tauri::command]
async fn export_sales(
    app: AppHandle,
    month: String,
    path: PathBuf,
    currency: Option<Currency>,
) -> Result<i64> {
    tauri::async_runtime::spawn_blocking(move || {
        store::export_csv(
            &*app.state::<Workstation>().lock()?,
            &month,
            &path,
            currency.unwrap_or_default(),
        )
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn backup_workspace(app: AppHandle, path: PathBuf) -> Result<()> {
    tauri::async_runtime::spawn_blocking(move || {
        store::snapshot(&*app.state::<Workstation>().lock()?, &path)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn restore_workspace(app: AppHandle, path: PathBuf) -> Result<()> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<Workstation>();
        let mut db = state.lock()?;
        store::restore(&mut db, &path, &state.directory)?;
        drop(db);
        notify_workspace(&app);
        Ok(())
    })
    .await
    .map_err(|error| error.to_string())?
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let directory = app.path().app_local_data_dir()?;
            #[cfg(debug_assertions)]
            let directory =
                std::env::var_os("ASCEL_BILL_TEST_DATA").map_or(directory, PathBuf::from);
            std::fs::create_dir_all(&directory)?;
            let db =
                store::open(&directory.join("workspace.sqlite")).map_err(std::io::Error::other)?;
            app.manage(Workstation {
                db: Mutex::new(db),
                directory,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            workspace,
            state_options,
            save_business,
            update_business,
            save_print_settings,
            set_usd_option,
            find_customer_contact,
            import_customer_csv,
            list_items,
            list_customers,
            save_item,
            update_stock,
            save_customer,
            selected_items,
            selected_customer,
            save_draft,
            get_draft,
            review_invoice,
            issue_invoice,
            get_invoice,
            invoice_register,
            dashboard,
            overview,
            print_invoice,
            export_sales,
            backup_workspace,
            restore_workspace
        ])
        .run(tauri::generate_context!())
        .expect("Could not start Ascel Bill");
}
