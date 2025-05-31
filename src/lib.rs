use std::{env, fs};

use zed::LanguageServerId;
use zed_extension_api::{self as zed, Result};

const LSP_SERVER_PATH: &str = "node_modules/stimulus-language-server/out/server.js";
const MCP_SERVER_PATH: &str = "node_modules/stimulus-language-server/out/mcp-server/server.js";
const PACKAGE_NAME: &str = "stimulus-language-server";

struct StimulusExtension {
    did_find_server: bool,
}

impl StimulusExtension {
    fn server_exists(&self) -> bool {
        fs::metadata(LSP_SERVER_PATH).is_ok_and(|stat| stat.is_file())
    }

    fn server_script_path(
        &mut self,
        language_server_id: &LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<String> {
        let server_exists = self.server_exists();
        if self.did_find_server && server_exists {
            return Ok(LSP_SERVER_PATH.to_string());
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let version = zed::npm_package_latest_version(PACKAGE_NAME)?;

        if !server_exists
            || zed::npm_package_installed_version(PACKAGE_NAME)?.as_ref() != Some(&version)
        {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            let result = zed::npm_install_package(PACKAGE_NAME, &version);
            match result {
                Ok(()) => {
                    if !self.server_exists() {
                        Err(format!(
                                    "installed package '{PACKAGE_NAME}' did not contain expected path '{LSP_SERVER_PATH}'",
                                ))?;
                    }
                }
                Err(error) => {
                    if !self.server_exists() {
                        Err(error)?;
                    }
                }
            }
        }

        self.did_find_server = true;
        Ok(LSP_SERVER_PATH.to_string())
    }
}

impl zed::Extension for StimulusExtension {
    fn new() -> Self {
        Self {
            did_find_server: false,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let server_path = self.server_script_path(language_server_id, worktree)?;

        Ok(zed::Command {
            command: zed::node_binary_path()?,
            args: vec![
                env::current_dir()
                    .unwrap()
                    .join(&server_path)
                    .to_string_lossy()
                    .to_string(),
                "--stdio".to_string(),
            ],
            env: Default::default(),
        })
    }

    fn context_server_command(
        &mut self,
        _context_server_id: &zed_extension_api::ContextServerId,
        _project: &zed_extension_api::Project,
    ) -> Result<zed_extension_api::Command> {
        Ok(zed_extension_api::Command {
            command: zed::node_binary_path()?,
            args: vec![
                env::current_dir()
                    .unwrap()
                    .join(&MCP_SERVER_PATH)
                    .to_string_lossy()
                    .to_string(),
                "--stdio".to_string(),
            ],
            env: Default::default(),
        })
    }
}

zed::register_extension!(StimulusExtension);

#[cfg(test)]
mod tests {
    use super::*;
    use zed_extension_api::Extension;

    #[test]
    fn test_new_extension_initial_state() {
        let ext = StimulusExtension::new();
        assert!(
            !ext.did_find_server,
            "A new extension instance should have did_find_server as false by default."
        );
    }
}
