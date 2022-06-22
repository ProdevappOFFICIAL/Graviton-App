use crate::extensions::base::ExtensionInfo;
use crate::extensions::manager::{ExtensionsManager, LoadedExtension};
use crate::filesystems::{Filesystem, LocalFilesystem};
use crate::messaging::ClientMessages;
pub use crate::state_persistors::memory::MemoryPersistor;
use crate::state_persistors::Persistor;
use crate::{Errors, ExtensionErrors, LanguageServer, ManifestInfo};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::warn;

use super::StateData;

/// A state is like a small configuration, like a profile
#[derive(Clone)]
pub struct State {
    pub filesystems: HashMap<String, Arc<Mutex<Box<dyn Filesystem + Send>>>>,
    pub extensions_manager: ExtensionsManager,
    pub persistor: Option<Arc<Mutex<Box<dyn Persistor + Send>>>>,
    pub data: StateData,
    pub tokens: Vec<String>,
    pub language_servers: HashMap<String, LanguageServer>,
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("State")
            .field("opened_tabs", &self.data.views)
            .field("id", &self.data.id)
            .finish()
    }
}

impl Default for State {
    /// The default constructor will include:
    /// - LocalFilesystem
    ///
    /// But will not persist the state
    fn default() -> Self {
        let mut filesystems = HashMap::new();

        // Support the local filesystem by default
        let local_fs: Box<dyn Filesystem + Send> = Box::new(LocalFilesystem::new());
        filesystems.insert("local".to_string(), Arc::new(Mutex::new(local_fs)));

        Self {
            data: StateData::default(),
            filesystems,
            extensions_manager: ExtensionsManager::default(),
            tokens: Vec::new(),
            persistor: None,
            language_servers: HashMap::new(),
        }
    }
}

impl State {
    pub fn new(
        id: u8,
        extensions_manager: ExtensionsManager,
        mut persistor: Box<dyn Persistor + Send>,
    ) -> Self {
        // Retrieve opened tabs from the persistor
        let state = persistor.load();

        State {
            data: StateData { id, ..state },
            extensions_manager,
            persistor: Some(Arc::new(Mutex::new(persistor))),
            ..Default::default()
        }
    }

    /// Retrieve the specified filesystem by the given name
    pub fn get_fs_by_name(
        &self,
        filesystem: &str,
    ) -> Option<Arc<Mutex<Box<dyn Filesystem + Send>>>> {
        return self.filesystems.get(filesystem).cloned();
    }

    // Check if the state can be used with the specified token
    pub fn has_token(&self, token: &str) -> bool {
        self.tokens.contains(&token.to_owned())
    }

    /// Run all the extensions in the manager
    pub async fn run_extensions(&self) {
        for ext in &self.extensions_manager.extensions {
            if let LoadedExtension::ExtensionInstance { plugin, .. } = ext {
                let mut ext_plugin = plugin.lock().await;
                ext_plugin.unload();
                ext_plugin.init();
            }
        }
    }

    /// Notify a specific extension about a perticular message
    pub fn notify_extension(&self, extension_id: String, message: ClientMessages) {
        for ext in &self.extensions_manager.extensions {
            if let LoadedExtension::ExtensionInstance {
                plugin, parent_id, ..
            } = ext
            {
                if parent_id == &extension_id {
                    let ext_plugin = plugin.clone();
                    let message = message.clone();
                    tokio::spawn(async move {
                        let mut ext_plugin = ext_plugin.lock().await;
                        ext_plugin.notify(message.clone());
                    });
                }
            }
        }
    }

    /// Notify all the extensions in a state about a message, asynchronously and independently
    pub fn notify_extensions(&self, message: ClientMessages) {
        for ext in &self.extensions_manager.extensions {
            if let LoadedExtension::ExtensionInstance { plugin, .. } = ext {
                let ext_plugin = plugin.clone();
                let message = message.clone();
                tokio::spawn(async move {
                    let mut ext_plugin = ext_plugin.lock().await;
                    ext_plugin.notify(message.clone());
                });
            }
        }
    }

    /// Try to retrieve info about a perticular loaded extension
    pub fn get_ext_info_by_id(&self, ext_id: &str) -> Result<ManifestInfo, Errors> {
        let extensions = &self.extensions_manager.extensions;
        let result = extensions.iter().find_map(|extension| {
            if let LoadedExtension::ManifestFile { manifest } = extension {
                if manifest.info.extension.id == ext_id {
                    Some(manifest.info.clone())
                } else {
                    None
                }
            } else if let LoadedExtension::ManifestBuiltin { info, .. } = extension {
                if info.extension.id == ext_id {
                    Some(info.clone())
                } else {
                    None
                }
            } else {
                None
            }
        });

        result.ok_or(Errors::Ext(ExtensionErrors::ExtensionNotFound))
    }

    /// Try to retrieve info about a perticular loaded extension
    pub fn get_ext_run_info_by_id(&self, ext_id: &str) -> Result<ExtensionInfo, Errors> {
        let extensions = &self.extensions_manager.extensions;
        let result = extensions.iter().find_map(|extension| {
            if let LoadedExtension::ExtensionInstance { info, .. } = extension {
                if info.id == ext_id {
                    Some(info.clone())
                } else {
                    None
                }
            } else {
                None
            }
        });

        result.ok_or(Errors::Ext(ExtensionErrors::ExtensionNotFound))
    }

    /// Return the list of loaded extensions
    pub fn get_ext_list_by_id(&self) -> Vec<String> {
        let extensions = &self.extensions_manager.extensions;

        extensions
            .iter()
            .filter_map(|extension| {
                if let LoadedExtension::ManifestBuiltin { info, .. } = extension {
                    Some(info.extension.id.to_string())
                } else if let LoadedExtension::ManifestFile { manifest } = extension {
                    Some(manifest.info.extension.id.to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>()
    }

    // Merge a new state data
    pub async fn update(&mut self, new_data: StateData) {
        let mut any_diff = false;

        if self.data.views != new_data.views {
            any_diff = true;
        }

        if self.data.commands != new_data.commands {
            any_diff = true;
        }

        if let Some(persistor) = &self.persistor {
            // Only save it if there has been any mutation in the state data
            if any_diff {
                persistor.lock().await.save(&self.data);
            }
        } else {
            warn!(
                "Persistor not found for State by id <{}>, could not save",
                self.data.id
            );
        }
    }

    // Register a new language server
    pub async fn register_language_servers(
        &mut self,
        language_servers: HashMap<String, LanguageServer>,
    ) {
        self.language_servers.extend(language_servers);
    }

    // Register a new language server
    pub async fn get_all_language_servers(&self) -> Vec<LanguageServer> {
        self.language_servers
            .values()
            .cloned()
            .collect::<Vec<LanguageServer>>()
    }
}

// NOTE: It would be interesting to implement https://doc.rust-lang.org/std/ops/trait.AddAssign.html
// So it's easier to merge 2 states, old + new

#[cfg(test)]
mod tests {

    use crate::extensions::base::{Extension, ExtensionInfo};
    use crate::extensions::manager::ExtensionsManager;
    use crate::messaging::ClientMessages;
    use crate::states::MemoryPersistor;

    use super::State;

    fn get_sample_extension_info() -> ExtensionInfo {
        ExtensionInfo {
            id: "sample".to_string(),
            name: "sample".to_string(),
        }
    }

    fn get_sample_extension() -> Box<dyn Extension + Send> {
        struct SampleExtension;

        impl Extension for SampleExtension {
            fn get_info(&self) -> ExtensionInfo {
                get_sample_extension_info()
            }

            fn init(&mut self) {
                todo!()
            }

            fn unload(&mut self) {
                todo!()
            }

            fn notify(&mut self, _message: ClientMessages) {
                todo!()
            }
        }

        Box::new(SampleExtension)
    }

    #[test]
    fn get_info() {
        let mut manager = ExtensionsManager::default();
        manager.register("sample", get_sample_extension());
        let test_state = State::new(0, manager, Box::new(MemoryPersistor::new()));

        let ext_info = test_state.get_ext_run_info_by_id("sample");
        assert!(ext_info.is_ok());

        let ext_info = ext_info.unwrap();
        assert_eq!(get_sample_extension_info(), ext_info);
    }
}