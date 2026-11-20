//! P0's read-only launcher core. It does not launch the game, write instance metadata or
//! initialize a renderer. JNI is an adapter around `dispatch`; host tests use the same function.
use horizon_client_core::mod_settings::ModSettings;
use horizon_mods::{Package, index::native};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[cfg(target_os = "android")]
mod jni;
#[cfg(test)]
mod tests;

pub const SCHEMA: u32 = 1;
pub const MAX_REQUEST_BYTES: usize = 64 << 10;

#[derive(Debug, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case", deny_unknown_fields)]
enum Request {
    Ping,
    InspectInstance {
        #[serde(rename = "instanceId")]
        instance_id: String,
        #[serde(rename = "gameRevision")]
        game_revision: String,
        #[serde(rename = "nativeHooksApi")]
        native_hooks_api: Option<u32>,
        #[serde(rename = "instanceDir")]
        instance_dir: PathBuf,
    },
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Error {
    pub code: &'static str,
    pub detail: String,
    pub fields: Vec<Field>,
}

impl Error {
    fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
            fields: Vec::new(),
        }
    }

    fn field(mut self, name: &str, value: impl Into<String>) -> Self {
        self.fields.push(Field {
            name: name.to_owned(),
            value: value.into(),
        });
        self
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Pong {
    protocol: &'static str,
    read_only: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InstanceDraft {
    pub schema: u32,
    pub instance_id: String,
    pub game_revision: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native_hooks_api: Option<u32>,
    pub native_build: Option<NativeBuild>,
    pub mods: Vec<ModSnapshot>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NativeBuild {
    pub present: bool,
    pub build_key: String,
    pub revision_matches: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModSnapshot {
    pub id: String,
    pub name: String,
    pub version: String,
    pub kind: &'static str,
    pub file: String,
    pub sha256: Option<String>,
    pub size: u64,
    pub selected: bool,
    pub build_required: bool,
    pub side: Option<&'static str>,
    pub namespaces: Vec<String>,
    pub game: Option<String>,
    pub horizon: Option<String>,
    pub mixins: Option<String>,
    pub hooks: Option<u32>,
    pub problem: Option<Error>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Value {
    Pong(Pong),
    Instance(InstanceDraft),
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Response {
    schema: u32,
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Error>,
}

impl From<Result<Value, Error>> for Response {
    fn from(result: Result<Value, Error>) -> Self {
        match result {
            Ok(value) => Self {
                schema: SCHEMA,
                ok: true,
                result: Some(value),
                error: None,
            },
            Err(error) => Self {
                schema: SCHEMA,
                ok: false,
                result: None,
                error: Some(error),
            },
        }
    }
}

/// Handles one bounded UTF-8 JSON request. All failures are data; no panic crosses the JNI boundary.
pub fn dispatch(request: &[u8]) -> String {
    let response = || -> Result<Value, Error> {
        if request.len() > MAX_REQUEST_BYTES {
            return Err(Error::new("invalid_request", "request exceeds 65536 bytes"));
        }
        let text = std::str::from_utf8(request)
            .map_err(|_| Error::new("invalid_request", "request is not UTF-8"))?;
        let request: Request = serde_json::from_str(text)
            .map_err(|error| Error::new("invalid_request", error.to_string()))?;
        match request {
            Request::Ping => Ok(Value::Pong(Pong {
                protocol: "rustling.launcher.ping.v1",
                read_only: true,
            })),
            Request::InspectInstance {
                instance_id,
                game_revision,
                native_hooks_api,
                instance_dir,
            } => inspect(
                &instance_id,
                &game_revision,
                native_hooks_api,
                &instance_dir,
            )
            .map(Value::Instance),
        }
    }();
    serde_json::to_string(&Response::from(response))
        .unwrap_or_else(|_| r#"{"schema":1,"ok":false,"error":{"code":"internal_error","detail":"response serialization failed","fields":[]}}"#.into())
}

fn io_error(code: &'static str, path: &Path, error: io::Error) -> Error {
    Error::new(code, format!("{}: {error}", path.display()))
}

fn inspect(
    instance_id: &str,
    game_revision: &str,
    native_hooks_api: Option<u32>,
    root: &Path,
) -> Result<InstanceDraft, Error> {
    if instance_id.is_empty()
        || instance_id.len() > 128
        || !instance_id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_:-".contains(&b))
    {
        return Err(Error::new(
            "invalid_instance",
            "instanceId must be 1-128 lowercase ASCII letters, digits, _, : or -",
        ));
    }
    if !native::revision(game_revision) {
        return Err(Error::new(
            "invalid_instance",
            "gameRevision must be an exact 40-character lowercase Git revision",
        ));
    }
    if native_hooks_api == Some(0) {
        return Err(Error::new(
            "invalid_instance",
            "nativeHooksApi must be positive",
        ));
    }
    if !root.is_absolute() {
        return Err(Error::new(
            "invalid_instance",
            "instanceDir must be absolute",
        ));
    }
    let metadata = fs::metadata(root).map_err(|e| io_error("instance_unreadable", root, e))?;
    if !metadata.is_dir() {
        return Err(Error::new(
            "instance_unreadable",
            "instanceDir is not a directory",
        ));
    }
    let selected = match fs::read_to_string(root.join("horizon-mods.txt")) {
        Ok(text) => ModSettings::parse(&text),
        Err(error) if error.kind() == io::ErrorKind::NotFound => ModSettings::default(),
        Err(error) => return Err(io_error("settings_unreadable", root, error)),
    };
    let mut files = Vec::new();
    for (folder, kind) in [
        ("horizon-mods", "content"),
        ("horizon-native-mods", "native"),
    ] {
        let dir = root.join(folder);
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(io_error("instance_unreadable", &dir, error)),
        };
        for entry in entries {
            let entry = entry.map_err(|e| io_error("instance_unreadable", &dir, e))?;
            let path = entry.path();
            let native = kind == "native";
            let supported = if native {
                path.is_file() && path.extension().is_some_and(|ext| ext == "hnmod")
            } else {
                path.is_dir() && path.join("horizon-mod.toml").is_file()
                    || path.is_file() && path.extension().is_some_and(|ext| ext == "hmod")
            };
            if supported {
                files.push((path, native));
            }
        }
    }
    files.sort_by(|(a, _), (b, _)| a.cmp(b));
    let mut namespaces: BTreeMap<String, String> = BTreeMap::new();
    let native_build = native_build(root, game_revision)?;
    let mut mods = Vec::new();
    for (path, is_native) in files {
        let mut mod_ = snapshot(&path, &selected, is_native, native_build.as_ref());
        if mod_.problem.is_none() {
            if is_native && mod_.horizon.as_deref() != Some(game_revision) && mod_.hooks.is_none() {
                mod_.problem = Some(Error::new(
                    "revision_mismatch",
                    "native mod pins a different Horizon revision and has no stable hook contract",
                ));
            }
            if is_native
                && mod_
                    .hooks
                    .zip(native_hooks_api)
                    .is_some_and(|(needed, actual)| needed > actual)
            {
                mod_.problem = Some(
                    Error::new("hook_api_unsupported", "native mod needs a newer hook API")
                        .field(
                            "requiredHooksApi",
                            mod_.hooks.unwrap_or_default().to_string(),
                        )
                        .field(
                            "gameHooksApi",
                            native_hooks_api.unwrap_or_default().to_string(),
                        ),
                );
            }
        }
        if mod_.selected && mod_.problem.is_none() && !is_native {
            for namespace in &mod_.namespaces {
                if let Some(owner) = namespaces.insert(namespace.clone(), mod_.id.clone())
                    && owner != mod_.id
                {
                    mod_.problem = Some(
                        Error::new(
                            "namespace_conflict",
                            "another enabled content mod owns this namespace",
                        )
                        .field("namespace", namespace)
                        .field("otherModId", owner),
                    );
                }
            }
        }
        mods.push(mod_);
    }
    Ok(InstanceDraft {
        schema: SCHEMA,
        instance_id: instance_id.to_owned(),
        game_revision: game_revision.to_owned(),
        native_hooks_api,
        native_build,
        mods,
    })
}

fn native_build(root: &Path, game_revision: &str) -> Result<Option<NativeBuild>, Error> {
    #[derive(Deserialize)]
    struct Build {
        key: String,
        commit: String,
    }
    let pointer = root.join("files/native/current");
    let key = match fs::read_to_string(&pointer) {
        Ok(key) => key,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io_error("build_unreadable", &pointer, error)),
    };
    let key = key.trim();
    if key.is_empty()
        || key.len() > 128
        || !key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
    {
        return Err(Error::new(
            "build_invalid",
            "files/native/current is not a plain build key",
        ));
    }
    let path = root.join("files/native").join(key).join("build.json");
    let text = fs::read_to_string(&path).map_err(|e| io_error("build_unreadable", &path, e))?;
    let build: Build = serde_json::from_str(&text)
        .map_err(|error| Error::new("build_invalid", format!("{}: {error}", path.display())))?;
    if build.key != key {
        return Err(Error::new(
            "build_invalid",
            "build.json key differs from files/native/current",
        ));
    }
    Ok(Some(NativeBuild {
        present: true,
        build_key: key.to_owned(),
        revision_matches: build.commit == game_revision,
    }))
}

fn snapshot(
    path: &Path,
    selected: &ModSettings,
    is_native: bool,
    native_build: Option<&NativeBuild>,
) -> ModSnapshot {
    let file = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let fallback = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let bytes = path.is_file().then(|| fs::read(path));
    let (id, name, version, side, namespaces, game, horizon, mixins, hooks, mut problem) =
        if is_native {
            match native::Bundle::read(path) {
                Ok(bundle) => {
                    let m = bundle.manifest;
                    (
                        m.id.clone(),
                        m.id,
                        m.version,
                        None,
                        Vec::new(),
                        Some(m.game),
                        Some(m.horizon),
                        Some(m.mixins),
                        m.hooks,
                        None,
                    )
                }
                Err(error) => (
                    fallback.clone(),
                    fallback.clone(),
                    String::new(),
                    None,
                    Vec::new(),
                    None,
                    None,
                    None,
                    None,
                    Some(Error::new("mod_invalid", error)),
                ),
            }
        } else {
            match Package::read(path) {
                Ok(package) => {
                    let m = package.manifest;
                    let name = format!("{} {}", m.name, m.version);
                    let namespaces = m.owned_namespaces().map(str::to_owned).collect();
                    (
                        m.id,
                        name,
                        m.version,
                        Some(side_name(m.side)),
                        namespaces,
                        None,
                        None,
                        None,
                        None,
                        None,
                    )
                }
                Err(error) => (
                    fallback.clone(),
                    fallback.clone(),
                    String::new(),
                    None,
                    Vec::new(),
                    None,
                    None,
                    None,
                    None,
                    Some(Error::new("mod_invalid", error)),
                ),
            }
        };
    let build_required = is_native
        && !selected.disabled.contains(&id)
        && native_build.is_none_or(|build| !build.revision_matches);
    let (sha256, size) = match bytes {
        Some(Ok(bytes)) => (Some(hex(&Sha256::digest(&bytes))), bytes.len() as u64),
        Some(Err(error)) => {
            problem = Some(io_error("mod_unreadable", path, error));
            (None, 0)
        }
        None => (None, 0),
    };
    ModSnapshot {
        selected: if is_native {
            !selected.disabled.contains(&id)
        } else {
            selected.enabled.contains(&id)
        },
        build_required,
        id,
        name,
        version,
        kind: if is_native { "native" } else { "content" },
        file,
        sha256,
        size,
        side,
        namespaces,
        game,
        horizon,
        mixins,
        hooks,
        problem,
    }
}

fn side_name(side: horizon_mods::Side) -> &'static str {
    match side {
        horizon_mods::Side::Client => "client",
        horizon_mods::Side::Server => "server",
        horizon_mods::Side::Both => "both",
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut out, byte| {
            let _ = write!(out, "{byte:02x}");
            out
        })
}
