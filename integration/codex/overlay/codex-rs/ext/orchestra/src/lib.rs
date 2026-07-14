mod tool;

use codex_core::ThreadManager;
use codex_extension_api::ExtensionRegistryBuilder;
use std::sync::{Arc, Weak};

pub fn install(
    registry: &mut ExtensionRegistryBuilder<codex_core::config::Config>,
    thread_manager: Weak<ThreadManager>,
) {
    registry.tool_contributor(Arc::new(tool::OrchestraTools::new(thread_manager)));
}
