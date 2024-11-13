// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use web_sys::{js_sys, wasm_bindgen::JsValue};

use crate::wallet::Error;

use super::StorageAdapter;

/// The storage id.
pub const STORAGE_ID: &str = "Wasm";

/// Wasm storage adapter using the browser local storage
#[derive(Debug)]
pub struct WasmAdapter {
    /// field used just to make it impossible to create a [`WasmAdapter`] without using the `new` function.
    _ignored: (),
}

const PROBE_KEY: &str = "iota-sdk-availability-test";
const PROBE_VALUE: &str = "local storage is available!";

impl WasmAdapter {
    pub fn new() -> crate::wallet::Result<Self> {
        // just do a quick check to see if the API is available!

        // do a write-read cycle, and if any of them fail, wrap the error and return it

        let storage = Self::storage()?;

        let out = || -> Result<bool, JsValue> {
            storage.set_item(PROBE_KEY, PROBE_VALUE)?;
            let read = storage.get_item(PROBE_KEY)?;
            let _ = storage.remove_item(PROBE_KEY);
            Ok(read.is_some_and(|v| v == PROBE_VALUE))
        }();

        let matched = out.map_err(|e| Error::Storage(format!("localStorage probe check failed with error: {e:?}")))?;

        if !matched {
            return Err(Error::Storage(
                "localStorage probe check failed: written and read values do not match!".to_string(),
            ));
        }

        Ok(Self { _ignored: () })
    }

    /// Use reflection instead of the `window` object to get hold of a reference to the local storage API.
    /// This makes sure we can utilize it when running in nodejs with a mocked global
    /// `window.localStorage` object.
    fn storage() -> crate::wallet::Result<web_sys::Storage> {
        let window_obj = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("window"))
            .map_err(|e| Error::Storage(format!("no window object found: {e:?}")))?;

        let local_storage_obj = js_sys::Reflect::get(&window_obj, &JsValue::from_str("localStorage"))
            .map_err(|e| Error::Storage(format!("no window.localStorage object found: {e:?}")))?;

        let storage = web_sys::Storage::try_from(local_storage_obj)
            .map_err(|e| Error::Storage(format!("window.localStorage should be web_sys::Storage: {e}")))?;
        Ok(storage)
    }
}

#[async_trait::async_trait]
impl StorageAdapter for WasmAdapter {
    type Error = crate::wallet::Error;

    /// Gets the record associated with the given key from the storage.
    async fn get_bytes(&self, key: &str) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(Self::storage()?
            .get_item(key)
            .map_err(|e| Error::Storage(format!("get_item error: {e:?}")))?
            .map(|s| s.into_bytes()))
    }

    /// Saves or updates a record on the storage.
    async fn set_bytes(&self, key: &str, record: &[u8]) -> Result<(), Self::Error> {
        // We always store valid UTF8 JSON, so this does not loose any information
        let record = String::from_utf8_lossy(record);

        Self::storage()?
            .set_item(key, &record)
            .map_err(|e| Error::Storage(format!("set_item error: {e:?}")))
    }

    /// Removes a record from the storage.
    async fn delete(&self, key: &str) -> crate::wallet::Result<()> {
        Self::storage()?
            .delete(key)
            .map_err(|e| Error::Storage(format!("delete error: {e:?}")))
    }
}
