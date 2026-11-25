# Desktop stack and low-end delivery

## Which maintained desktop stack best fits AscelBill?

### Takeaway

Use Tauri 2 with a bundled Svelte 5, TypeScript and Vite UI, a Rust application core and SQLite via rusqlite. Confirm Windows installation, memory use, PDF fidelity and physical printing in a small post-approval build before committing the rest of the MVP.

### Cited Findings

- Tauri maps UI to WebView2 on Windows and WKWebView on macOS. The core release log records 2.12.7 on 18 November 2026. Its GitHub organisation showed around 113,000 stars for the core repository in the November snapshot. Release activity and stars are evidence of ecosystem interest, with no direct implication for product performance or installed market share. [Tauri webview mapping](https://v2.tauri.app/reference/webview-versions/), [Tauri release log](https://tauri.app/release/tauri/), [Tauri GitHub](https://github.com/orgs/tauri-apps/repositories)
- Electron embeds Chromium and Node.js and supplies consistent rendering and a direct print and PDF API. Its official release feed showed 45.5.0 on 24 November 2026. Electron v44 and higher only publish 64-bit builds. The v43 line with Windows x86 ends in January 2027. [Electron introduction](https://www.electronjs.org/docs/latest/), [Electron releases](https://releases.electronjs.org/), [Electron webContents](https://www.electronjs.org/docs/latest/api/web-contents#contentsprinttopdfoptions), [Electron breaking changes](https://www.electronjs.org/docs/latest/breaking-changes)
- Wails v3 became stable for desktop on 26 October 2026. It uses Go with system WebViews, covers Windows amd64/ARM64 and Intel/Apple Silicon Mac, and provides an updater. Its mobile support remains experimental. v3.1.0 followed on 1 November 2026. [Wails 3 release](https://wails.io/blog/wails-v3-released/), [Wails compatibility](https://v3.wails.io/status/), [Wails changelog](https://wails.io/changelog/)
- Flutter supports native desktop targets and mobile. Its current deployment matrix identifies Windows 10/11 x64 and ARM64 plus macOS 12 through 27, and warns that Intel Mac support is being phased out. macOS 26 Tahoe is Apple's last Intel release. Its use of Dart and its UI/painter ecosystem represents a separate product stack. [Flutter deployment matrix](https://docs.flutter.dev/reference/supported-platforms), [Flutter desktop](https://docs.flutter.dev/platform-integration/desktop)
- Avalonia is a .NET/XAML candidate with a Win32-based Windows backend. Its current matrix puts Windows 10 22H2 x64 and macOS 14/15 in best-effort Tier 2. Earlier Windows builds including x86 and macOS 13 or earlier are Tier 3, handled as commercial support. [Avalonia supported platforms](https://docs.avaloniaui.net/docs/supported-platforms)
- Browser storage has quotas and best-effort storage may be evicted, depending on browser settings and pressure. A PWA can be considered for companion access, while the billing machine uses a local desktop database with app-managed recoverability. [Browser storage quota and eviction](https://developer.mozilla.org/en-US/docs/Web/API/Storage_API/Storage_quotas_and_eviction_criteria)
- Tauri's own benchmark repository uses Hello World or constrained workloads on GitHub Actions runners. These can inform framework investigation but cannot establish expected AscelBill memory or speed on a low-end office PC. [Tauri benchmark methodology](https://github.com/tauri-apps/benchmark_results)

### Inferences

- Prefer Tauri for the initial balance of a light distributable, Rust domain logic, a familiar web UI and desktop packaging. Use a production build with local resources only, one main webview, paginated or virtualized result lists, and indexed SQLite queries. Avoid remote scripts/fonts in the packaged app.
- Electron is the fallback if a common renderer and mature print pipeline materially reduce a demonstrated printing or development issue and the full build meets the same hardware targets. Wails is the alternate system-WebView framework for an established Go team. Flutter becomes more compelling if native Android and iOS join the initial scope. Avalonia becomes more compelling for an established C# team with acceptable platform support tiers.
- Use a reusable invoice document model and a local, deterministic PDF generator with bundled, properly licensed fonts. Initiate ordinary printing through documented OS or WebView interfaces after tests. Verify wrapping, clipping, pagination, ₹ and local-script glyphs, actual paper margins and physical devices. Tauri's Rust print method has platform limits, while the docs state browser window.print() is available across platforms. Windows WebView2 also offers platform-specific printing and PDF APIs. [Tauri WebviewWindow print](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindow.html), [WebView2 printing](https://learn.microsoft.com/en-us/microsoft-edge/webview2/how-to/print)

### Gaps

- No actual AscelBill or competitor RSS, launch times or app installer have been measured on the target hardware. Share measurable targets as acceptance criteria.
- Team stack experience and the existence of reusable code are unverified. The supplied workspace was empty when inspected.

## How should device support and installation be scoped?

### Takeaway

Ship Windows x64 first with a defined 4 GB performance gate. Target current supported Windows 11 and separately test Windows 10 22H2 for legacy compatibility. Release a signed macOS build for Apple Silicon and eligible Intel Macs after testing both.

### Cited Findings

- Windows 10 22H2 mainstream updates ended 14 October 2025. Separate Extended Security Updates enrollment and some LTSC schedules apply. Microsoft documents Edge/WebView2 updates on 22H2 until at least October 2028. Microsoft Home/Pro lifecycle presently lists Windows 11 25H2, 26H1 and 26H2 as in service, and 24H2 ended October 2026 for those editions. [Windows 10 lifecycle](https://learn.microsoft.com/en-us/windows/release-health/release-information), [WebView2 lifecycle](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-support-lifecycle), [Windows 11 Home and Pro lifecycle](https://learn.microsoft.com/en-us/lifecycle/products/windows-11-home-and-pro)
- Tauri documents a standard Windows installer that downloads a WebView2 bootstrapper only if needed, against an offline installer carrying the evergreen WebView2 standalone installer at about 127 MB additional size. The fixed runtime option adds about 180 MB. Tauri documents x64, x86 and ARM64 build targets, with a native ARM64 binary using a 32-bit NSIS installer under emulation. These facts indicate technical possibilities, with OS, drivers and app QA still determining supported combinations. [Tauri Windows installer](https://v2.tauri.app/distribute/windows-installer/), [WebView2 runtime distribution](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution)
- Apple documents that macOS 27 Golden Gate runs on Apple Silicon. Its security release page lists Sequoia 15, Tahoe 26 and Golden Gate 27, and identifies specific Intel models eligible for Tahoe. Tauri documents a universal architecture build. Apple Developer Program membership for Developer ID signing is listed at US$99 per year or local equivalent. [Apple security releases](https://support.apple.com/en-us/100100), [Tauri universal build](https://tauri.app/distribute/app-store/), [Apple signing](https://developer.apple.com/developer-id/), [Apple price](https://developer.apple.com/programs/whats-included)
- Microsoft lists the organization territories for Azure Artifact Signing Public Trust without India. An Indian publisher should get an Authenticode quote from an eligible commercial provider or assess the Store channel. Tauri also supports signed application updates and HTTPS updater endpoints. [Microsoft territory list](https://learn.microsoft.com/en-us/azure/artifact-signing/quickstart), [Tauri Windows signing](https://v2.tauri.app/distribute/sign/windows/), [Tauri signed updates](https://v2.tauri.app/plugin/updater/)

### Inferences

- Suggested phase one matrix is supported Windows 11 x64 as primary, Windows 10 22H2 x64 compatibility on current WebView2 with applicable OS security servicing stated, and macOS 15/26/27 on eligible Intel or Apple Silicon with a notarized and stapled universal DMG. Re-evaluate OS servicing and architecture commitments at release. Use Windows ARM64 when the physical device and printer matrix confirms readiness. Treat Windows x86 as demand-gated because runtime/toolchain, dependency and device/driver support needs independent evidence. Defer Windows 7/8, Linux desktop and native mobile to demand validation.
- Deliver a compact standard Windows download and a larger signed USB-ready installer with evergreen runtime included. Test installation from a clean unconnected Windows 10 VM without WebView2, install from a standard account where applicable, and first launch and invoice/PDF/backup while offline. Verify installation failure messages and update behavior on reconnect.
- Proposed test hardware is a real 4 GB, dual-core Celeron/Pentium-class x64 Windows laptop at 1366×768 as well as a supported Windows 11 x64 system at 4 GB. Include SSD and a Windows 10 HDD lane if customer hardware warrants it. Run a real Intel Mac and Apple Silicon Mac. Include a driver-supported A4 USB printer and 80 mm thermal printer if the rollout targets retail.
- Suggested performance gates on the reference 4 GB Windows machine with a representative 50,000-invoice and 10,000-item seeded database are cold start within 4 s p95, item lookup within 200 ms p95, single invoice save within 350 ms p95, ten-line PDF within 2 s p95, and idle app process-tree private working set within 250 MB after 60 s. Measure at least 20 starts and 50 each of lookup/save/PDF, log build and WebView versions, and report observed distributions for each OS and disk class. Standard installer size objective 30 MB or less without WebView2. These are proposal values and hardware must be validated before making a compatibility or performance claim.
- Budgets should cover signing, lightweight release artifact storage, CI/build capacity, test machines and printers, GST rules maintenance and support. Base billing should have no required hosting workload per invoice. Confirm Azure eligibility or commercial certificate pricing before planning exact costs.

### Gaps

- Proportion of prospects on Windows 10/32-bit/ARM, Intel Mac ownership, printer models and desired interfaces should come from pilot intake. Exact Windows signing charge for the Indian publisher is quote- and vendor-dependent.
- Compatibility of the chosen PDF engine with Bengali, Hindi, and other complex scripts requires actual test templates and font shaping checks.

## How should one or many devices handle business data?

### Takeaway

Deliver offline single-device billing and recoverability in v1. Treat multi-device use as a staged feature: desktop builds and portable backups first, then optional read-only access, then controlled offline multi-writer sync after invoice integrity is proven.

### Cited Findings

- SQLite's WAL requires all processes using a database to be on one machine. SQLite's supported Online Backup API produces a consistent snapshot, and its docs warn of corruption risks from unsafe live copies and broken network locking. [SQLite WAL](https://sqlite.org/wal.html), [SQLite backup API](https://sqlite.org/backup.html), [SQLite corruption risks](https://sqlite.org/howtocorrupt.html)
- SQLite documents a rare WAL reset corruption bug fixed in 3.51.3 and later, with backports to 3.50.7 and 3.44.6. The application should verify the version actually linked at build/runtime. Rusqlite provides an idiomatic Rust wrapper. [SQLite WAL-reset fix](https://sqlite.org/wal.html#walresetbug), [rusqlite](https://docs.rs/rusqlite/latest/rusqlite/)
- Tauri has desktop and mobile targets, with platform-dependent native integrations. Shared shell or UI cannot alone supply synchronized accounting state. [What is Tauri?](https://v2.tauri.app/start/), [Tauri WebView versions](https://v2.tauri.app/reference/webview-versions/)

### Inferences

- Put the SQLite file in OS application data local storage. Use parameterized queries and transactional issue operations that commit immutable invoice headers and line items, tax breakdown, document sequence and related stock or payment effects together. Use integer paise for stored money, precise decimal or scaled integer calculations for quantity and tax, and a defined rounding policy. Record tax and party snapshots on issue and preserve audit history. Keep authority in Rust through named commands with narrow Tauri permissions.
- Backup via the SQLite snapshot API into an app-managed versioned archive with checksum and schema version. Keep rotating local backups and offer portable encrypted backups with a tested restore flow and external destination. Restores validate integrity and version before replacement, preserve current state for recovery, and require exclusive use. Offline restore on a second computer should be a migration rather than two independently editable copies.
- When optional sync is approved, use device identities, UUID records, a transactional outbox, idempotent writes, per-record versions, visible conflict handling, incremental pulls and encryption in transit. Keep immutable issued invoice and stock movement records, and add documented device/branch series or disjoint number ranges verified with the GST adviser before permitting multi-device offline issuance. Never synchronize or share the live SQLite file via OneDrive, WhatsApp or a network share. Consider a read-only web/mobile companion before simultaneous writes.
- Any govt filing/IRN/e-way bill connection should be separate from the local UI and activate when online or direct the user to the approved portal. License and update rules should preserve ordinary local billing during outages, with entitlements and admin updates handled separately.

### Gaps

- Exact conflict structure depends on whether pilot users need two active tills or merely home/store visibility, how many GSTINs they have, and the allocation rules approved by the compliance reviewer.
- Backup frequency, retention, encryption UX and cloud geography require discovery against pilot recovery needs and business requirements.
