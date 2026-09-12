use crate::protocol::{Result, TradeXError};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use zeroize::Zeroizing;

const SERVICE: &str = "com.tradex.model.credentials";

pub struct ModelKey(Zeroizing<String>);

impl ModelKey {
    pub fn new(value: String) -> Result<Self> {
        if value.is_empty()
            || value.len() > 512
            || !value.bytes().all(|byte| byte.is_ascii_graphic())
        {
            return Err(TradeXError::new("MODEL_KEY_INVALID"));
        }
        Ok(Self(Zeroizing::new(value)))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

pub trait ModelVault: Send + Sync {
    fn put_deepseek(&self, workspace_id: &str, key: &ModelKey) -> Result<()>;
    fn get_deepseek(&self, workspace_id: &str) -> Result<ModelKey>;
    fn remove_deepseek(&self, workspace_id: &str) -> Result<()>;
}

pub struct NativeModelVault;

#[cfg(target_os = "macos")]
impl ModelVault for NativeModelVault {
    fn put_deepseek(&self, workspace_id: &str, key: &ModelKey) -> Result<()> {
        security_framework::passwords::set_generic_password(SERVICE, workspace_id, key.0.as_bytes())
            .map_err(|_| TradeXError::new("MODEL_KEYCHAIN_STORE_FAILED"))
    }

    fn get_deepseek(&self, workspace_id: &str) -> Result<ModelKey> {
        let bytes = Zeroizing::new(
            security_framework::passwords::get_generic_password(SERVICE, workspace_id)
                .map_err(|_| TradeXError::new("MODEL_KEYCHAIN_MISSING"))?,
        );
        if bytes.len() > 4096 {
            return Err(TradeXError::new("MODEL_KEYCHAIN_MISSING"));
        }
        let value = String::from_utf8(bytes.to_vec())
            .map_err(|_| TradeXError::new("MODEL_KEYCHAIN_MISSING"))?;
        ModelKey::new(value).map_err(|_| TradeXError::new("MODEL_KEYCHAIN_MISSING"))
    }

    fn remove_deepseek(&self, workspace_id: &str) -> Result<()> {
        match security_framework::passwords::delete_generic_password(SERVICE, workspace_id) {
            Ok(()) => Ok(()),
            Err(error) if error.code() == -25300 => Ok(()),
            Err(_) => Err(TradeXError::new("MODEL_KEYCHAIN_DELETE_FAILED")),
        }
    }
}

#[cfg(not(target_os = "macos"))]
impl ModelVault for NativeModelVault {
    fn put_deepseek(&self, _: &str, _: &ModelKey) -> Result<()> {
        Err(TradeXError::new("MODEL_PLATFORM_UNSUPPORTED"))
    }

    fn get_deepseek(&self, _: &str) -> Result<ModelKey> {
        Err(TradeXError::new("MODEL_KEYCHAIN_MISSING"))
    }

    fn remove_deepseek(&self, _: &str) -> Result<()> {
        Err(TradeXError::new("MODEL_KEYCHAIN_DELETE_FAILED"))
    }
}

#[derive(Default)]
pub struct MemoryModelVault {
    keys: Arc<Mutex<HashMap<String, Zeroizing<String>>>>,
}

impl ModelVault for MemoryModelVault {
    fn put_deepseek(&self, workspace_id: &str, key: &ModelKey) -> Result<()> {
        self.keys
            .lock()
            .map_err(|_| TradeXError::new("MODEL_KEYCHAIN_STORE_FAILED"))?
            .insert(
                workspace_id.to_owned(),
                Zeroizing::new(key.as_str().to_owned()),
            );
        Ok(())
    }

    fn get_deepseek(&self, workspace_id: &str) -> Result<ModelKey> {
        let value = self
            .keys
            .lock()
            .map_err(|_| TradeXError::new("MODEL_KEYCHAIN_MISSING"))?
            .get(workspace_id)
            .map(|value| value.as_str().to_owned())
            .ok_or_else(|| TradeXError::new("MODEL_KEYCHAIN_MISSING"))?;
        ModelKey::new(value)
    }

    fn remove_deepseek(&self, workspace_id: &str) -> Result<()> {
        self.keys
            .lock()
            .map_err(|_| TradeXError::new("MODEL_KEYCHAIN_DELETE_FAILED"))?
            .remove(workspace_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_keys_are_validated_and_memory_vault_is_workspace_scoped() {
        assert!(ModelKey::new("".into()).is_err());
        assert!(ModelKey::new("contains whitespace".into()).is_err());
        let vault = MemoryModelVault::default();
        let first = ModelKey::new("first-key".into()).unwrap();
        let second = ModelKey::new("second-key".into()).unwrap();
        vault.put_deepseek("one", &first).unwrap();
        vault.put_deepseek("two", &second).unwrap();
        assert_eq!(vault.get_deepseek("one").unwrap().as_str(), "first-key");
        assert_eq!(vault.get_deepseek("two").unwrap().as_str(), "second-key");
        vault.remove_deepseek("one").unwrap();
        assert!(vault.get_deepseek("one").is_err());
        assert_eq!(vault.get_deepseek("two").unwrap().as_str(), "second-key");
    }
}

#[cfg(feature = "desktop")]
mod native_entry {
    use super::ModelKey;
    use crate::protocol::{Result, TradeXError};
    use objc2::{MainThreadMarker, MainThreadOnly};
    use objc2_app_kit::{
        NSAccessibility, NSAlert, NSEventModifierFlags, NSSecureTextField, NSTextField, NSView,
    };
    use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    };
    use tauri::AppHandle;

    static ENTRY_OPEN: AtomicBool = AtomicBool::new(false);

    struct EntryGuard;
    impl Drop for EntryGuard {
        fn drop(&mut self) {
            ENTRY_OPEN.store(false, Ordering::Release);
        }
    }

    pub fn capture(app: &AppHandle) -> Result<ModelKey> {
        if ENTRY_OPEN
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(TradeXError::new("MODEL_ENTRY_BUSY"));
        }
        let _guard = EntryGuard;
        let (send, receive) = mpsc::sync_channel(1);
        app.run_on_main_thread(move || {
            let _ = send.send(show());
        })
        .map_err(|_| TradeXError::new("MODEL_NATIVE_ENTRY_REQUIRED"))?;
        receive
            .recv()
            .map_err(|_| TradeXError::new("MODEL_ENTRY_CANCELLED"))?
    }

    fn show() -> Result<ModelKey> {
        let mtm = MainThreadMarker::new()
            .ok_or_else(|| TradeXError::new("MODEL_NATIVE_ENTRY_REQUIRED"))?;
        let alert = NSAlert::new(mtm);
        alert.setMessageText(&NSString::from_str("Configure DeepSeek"));
        alert.setInformativeText(&NSString::from_str("The API key goes directly to the macOS Keychain. It is never sent to the renderer or saved in the workspace. Press Command-Return to save, or Escape to cancel."));
        let save = alert.addButtonWithTitle(&NSString::from_str("Save key"));
        save.setKeyEquivalent(&NSString::from_str("\r"));
        save.setKeyEquivalentModifierMask(NSEventModifierFlags::Command);
        let cancel = alert.addButtonWithTitle(&NSString::from_str("Cancel"));
        cancel.setKeyEquivalent(&NSString::from_str("\u{1b}"));
        let view = NSView::initWithFrame(NSView::alloc(mtm), rect(0.0, 0.0, 420.0, 80.0));
        let label = NSTextField::labelWithString(&NSString::from_str("DeepSeek API key"), mtm);
        label.setFrame(rect(0.0, 50.0, 420.0, 22.0));
        view.addSubview(&label);
        let field = NSSecureTextField::initWithFrame(
            NSSecureTextField::alloc(mtm),
            rect(0.0, 18.0, 420.0, 28.0),
        );
        field.setAccessibilityLabel(Some(&NSString::from_str("DeepSeek API key")));
        view.addSubview(&field);
        unsafe {
            field.setNextKeyView(Some(&save));
            save.setNextKeyView(Some(&cancel));
            cancel.setNextKeyView(Some(&field));
        }
        alert.setAccessoryView(Some(&view));
        alert.window().setInitialFirstResponder(Some(&field));
        let result = if alert.runModal() == 1000 {
            ModelKey::new(field.stringValue().to_string())
                .map_err(|_| TradeXError::new("MODEL_KEY_INVALID"))
        } else {
            Err(TradeXError::new("MODEL_ENTRY_CANCELLED"))
        };
        field.setStringValue(&NSString::from_str(""));
        result
    }

    fn rect(x: f64, y: f64, width: f64, height: f64) -> NSRect {
        NSRect::new(NSPoint::new(x, y), NSSize::new(width, height))
    }
}

#[cfg(feature = "desktop")]
pub use native_entry::capture;
