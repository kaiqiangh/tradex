use crate::{
    protocol::{Result, TradeXError},
    provider_io::Credentials,
    providers::ProviderDefinition,
};
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSAccessibility, NSAlert, NSEventModifierFlags, NSSecureTextField, NSTextField, NSView,
};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc,
};

static ENTRY_OPEN: AtomicBool = AtomicBool::new(false);
struct EntryGuard;
impl Drop for EntryGuard {
    fn drop(&mut self) {
        ENTRY_OPEN.store(false, Ordering::Release);
    }
}

pub fn capture(app: &tauri::AppHandle, definition: &ProviderDefinition) -> Result<Credentials> {
    if ENTRY_OPEN
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Err(TradeXError::new("PROVIDER_ENTRY_BUSY"));
    }
    let _guard = EntryGuard;
    let (send, receive) = mpsc::sync_channel(1);
    let definition = definition.clone();
    app.run_on_main_thread(move || {
        let _ = send.send(show(&definition));
    })
    .map_err(|_| TradeXError::new("PROVIDER_NATIVE_ENTRY_REQUIRED"))?;
    receive
        .recv()
        .map_err(|_| TradeXError::new("PROVIDER_ENTRY_CANCELLED"))?
}

fn show(definition: &ProviderDefinition) -> Result<Credentials> {
    let mtm = MainThreadMarker::new()
        .ok_or_else(|| TradeXError::new("PROVIDER_NATIVE_ENTRY_REQUIRED"))?;
    let alert = NSAlert::new(mtm);
    alert.setMessageText(&NSString::from_str(&format!(
        "Connect {}",
        definition.display_name
    )));
    alert.setInformativeText(&NSString::from_str("Credentials go directly to macOS Keychain. TradeX will only read this account during connection testing. You will review permissions before confirming. Press Command-Return to test, or Escape to cancel."));
    let submit = alert.addButtonWithTitle(&NSString::from_str("Test Connection"));
    submit.setKeyEquivalent(&NSString::from_str("\r"));
    submit.setKeyEquivalentModifierMask(NSEventModifierFlags::Command);
    let cancel = alert.addButtonWithTitle(&NSString::from_str("Cancel"));
    cancel.setKeyEquivalent(&NSString::from_str("\u{1b}"));
    let height = definition.fields.len() as f64 * 86.0;
    let view = NSView::initWithFrame(NSView::alloc(mtm), rect(0.0, 0.0, 420.0, height));
    let mut fields = Vec::new();
    for (index, schema) in definition.fields.iter().enumerate() {
        let y = height - (index as f64 + 1.0) * 86.0;
        let label = NSTextField::labelWithString(&NSString::from_str(&schema.label), mtm);
        label.setFrame(rect(0.0, y + 50.0, 420.0, 22.0));
        view.addSubview(&label);
        let input = NSSecureTextField::initWithFrame(
            NSSecureTextField::alloc(mtm),
            rect(0.0, y + 20.0, 420.0, 28.0),
        );
        input.setAccessibilityLabel(Some(&NSString::from_str(&schema.label)));
        input.setToolTip(Some(&NSString::from_str(&schema.help_text)));
        view.addSubview(&input);
        fields.push(input);
    }
    // SAFETY: all views are retained for the entire modal session; AppKit owns the key-view loop afterward.
    unsafe {
        for pair in fields.windows(2) {
            pair[0].setNextKeyView(Some(&pair[1]));
        }
        if let Some(last) = fields.last() {
            last.setNextKeyView(Some(&submit));
        }
        submit.setNextKeyView(Some(&cancel));
        if let Some(first) = fields.first() {
            cancel.setNextKeyView(Some(first));
        }
    }
    alert.setAccessoryView(Some(&view));
    if let Some(first) = fields.first() {
        alert.window().setInitialFirstResponder(Some(first));
    }
    let result = loop {
        if alert.runModal() != 1000 {
            break Err(TradeXError::new("PROVIDER_ENTRY_CANCELLED"));
        }
        let values = fields
            .iter()
            .map(|field| field.stringValue().to_string())
            .collect();
        match Credentials::new(values) {
            Ok(credentials)=>break Ok(credentials),
            Err(_)=>alert.setInformativeText(&NSString::from_str("Each required field must contain 1–512 printable characters without spaces. Correct the secure fields or cancel. No credential has been saved. Command-Return tests; Escape cancels.")),
        }
    };
    for field in fields {
        field.setStringValue(&NSString::from_str(""));
    }
    result
}

fn rect(x: f64, y: f64, width: f64, height: f64) -> NSRect {
    NSRect::new(NSPoint::new(x, y), NSSize::new(width, height))
}
