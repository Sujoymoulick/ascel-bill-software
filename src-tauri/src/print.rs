use crate::model::{InvoiceTemplate, Result};
use tauri::WebviewWindow;

#[cfg(target_os = "macos")]
pub fn invoice(window: WebviewWindow, template: InvoiceTemplate) -> Result<()> {
    use objc2_app_kit::{NSPrintInfo, NSWindow};
    use objc2_foundation::{NSCopying, NSSize};
    use objc2_web_kit::WKWebView;

    let (width, height) = match template {
        InvoiceTemplate::Standard | InvoiceTemplate::Ledger => (210.0, 297.0),
        InvoiceTemplate::LaserA5 => (148.0, 210.0),
        InvoiceTemplate::LaserA6 => (105.0, 148.0),
    };
    let points_per_mm = 72.0 / 25.4;
    window
        .with_webview(move |platform| {
            // Tauri runs the callback on the main thread and owns both Cocoa handles.
            unsafe {
                let view = &*platform.inner().cast::<WKWebView>();
                let parent = &*platform.ns_window().cast::<NSWindow>();
                let settings = NSPrintInfo::sharedPrintInfo().copy();
                settings.setPaperSize(NSSize::new(width * points_per_mm, height * points_per_mm));
                settings.setTopMargin(0.0);
                settings.setBottomMargin(0.0);
                settings.setLeftMargin(0.0);
                settings.setRightMargin(0.0);
                settings.setHorizontallyCentered(false);
                settings.setVerticallyCentered(false);

                let operation = view.printOperationWithPrintInfo(&settings);
                operation.setCanSpawnSeparateThread(true);
                operation.runOperationModalForWindow_delegate_didRunSelector_contextInfo(
                    parent,
                    None,
                    None,
                    std::ptr::null_mut(),
                );
            }
        })
        .map_err(|error| format!("Could not open printing: {error}"))
}

#[cfg(not(target_os = "macos"))]
pub fn invoice(window: WebviewWindow, _template: InvoiceTemplate) -> Result<()> {
    window
        .eval("window.print()")
        .map_err(|error| format!("Could not open printing: {error}"))
}
