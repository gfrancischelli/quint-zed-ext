use zed_extension_api::{self as zed, Result, settings::LspSettings};

const SERVER_NAME: &str = "quint-language-server";
const INSTALL_HINT: &str =
    "`quint-language-server` not found on PATH. Install with: \
     npm install -g @informalsystems/quint-language-server";

struct QuintExtension {
    cached_binary_path: Option<String>,
}

impl QuintExtension {
    fn language_server_binary(&mut self, worktree: &zed::Worktree) -> Result<zed::Command> {
        let binary_settings = LspSettings::for_worktree(SERVER_NAME, worktree)
            .ok()
            .and_then(|s| s.binary);

        let args = binary_settings
            .as_ref()
            .and_then(|b| b.arguments.clone())
            .unwrap_or_else(|| vec!["--stdio".into()]);

        if let Some(path) = binary_settings.and_then(|b| b.path) {
            return Ok(zed::Command {
                command: path,
                args,
                env: Default::default(),
            });
        }

        if let Some(path) = self.cached_binary_path.clone() {
            return Ok(zed::Command {
                command: path,
                args,
                env: Default::default(),
            });
        }

        if let Some(path) = worktree.which(SERVER_NAME) {
            self.cached_binary_path = Some(path.clone());
            return Ok(zed::Command {
                command: path,
                args,
                env: Default::default(),
            });
        }

        Err(INSTALL_HINT.to_string())
    }
}

impl zed::Extension for QuintExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        self.language_server_binary(worktree)
    }

    fn language_server_workspace_configuration(
        &mut self,
        server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        LspSettings::for_worktree(server_id.as_ref(), worktree)
            .map(|s| s.settings)
    }

    fn language_server_initialization_options(
        &mut self,
        server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        LspSettings::for_worktree(server_id.as_ref(), worktree)
            .map(|s| s.initialization_options)
    }
}

zed::register_extension!(QuintExtension);
