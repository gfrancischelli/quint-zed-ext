use std::{env, fs};
use zed_extension_api::{
    self as zed, LanguageServerId, LanguageServerInstallationStatus, Result,
    set_language_server_installation_status, settings::LspSettings,
};

const SERVER_NAME: &str = "quint-language-server";
const PACKAGE_NAME: &str = "@informalsystems/quint-language-server";
const SERVER_PATH: &str = "node_modules/@informalsystems/quint-language-server/out/src/server.js";

struct QuintExtension {
    cached_absolute_server_path: Option<String>,
}

impl QuintExtension {
    fn server_exists(&self) -> bool {
        fs::metadata(SERVER_PATH).is_ok_and(|stat| stat.is_file())
    }

    fn install_if_needed(&mut self, id: &LanguageServerId) -> Result<()> {
        set_language_server_installation_status(
            id,
            &LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let latest = zed::npm_package_latest_version(PACKAGE_NAME)?;
        let installed = zed::npm_package_installed_version(PACKAGE_NAME)?;

        if !self.server_exists() || installed.as_deref() != Some(&latest) {
            set_language_server_installation_status(
                id,
                &LanguageServerInstallationStatus::Downloading,
            );
            match zed::npm_install_package(PACKAGE_NAME, &latest) {
                Ok(()) => {
                    if !self.server_exists() {
                        return Err(format!(
                            "installed package '{PACKAGE_NAME}' did not contain expected path '{SERVER_PATH}'"
                        ));
                    }
                }
                Err(err) => {
                    if !self.server_exists() {
                        return Err(err);
                    }
                }
            }
        }
        Ok(())
    }
}

impl zed::Extension for QuintExtension {
    fn new() -> Self {
        Self {
            cached_absolute_server_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let binary_settings = LspSettings::for_worktree(SERVER_NAME, worktree)
            .ok()
            .and_then(|s| s.binary);

        if let Some(settings) = binary_settings {
            if let Some(path) = settings.path {
                return Ok(zed::Command {
                    command: path,
                    args: settings.arguments.unwrap_or_else(|| vec!["--stdio".into()]),
                    env: Default::default(),
                });
            }
        }

        let server_path = if let Some(cached) = self
            .cached_absolute_server_path
            .as_ref()
            .filter(|_| self.server_exists())
        {
            cached.clone()
        } else {
            self.install_if_needed(id)?;
            let absolute = env::current_dir()
                .map_err(|e| format!("couldn't resolve extension work dir: {e}"))?
                .join(SERVER_PATH)
                .to_string_lossy()
                .to_string();
            self.cached_absolute_server_path = Some(absolute.clone());
            absolute
        };

        Ok(zed::Command {
            command: zed::node_binary_path()?,
            args: vec![server_path, "--stdio".into()],
            env: Default::default(),
        })
    }

    fn language_server_workspace_configuration(
        &mut self,
        server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        LspSettings::for_worktree(server_id.as_ref(), worktree).map(|s| s.settings)
    }

    fn language_server_initialization_options(
        &mut self,
        server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        LspSettings::for_worktree(server_id.as_ref(), worktree)
            .map(|s| s.initialization_options)
    }
}

zed::register_extension!(QuintExtension);
