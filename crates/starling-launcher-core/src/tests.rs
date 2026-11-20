use super::*;
use serde_json::{Value, json};
use std::time::{SystemTime, UNIX_EPOCH};

const REVISION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const OLD_REVISION: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const MIXINS: &str = "cccccccccccccccccccccccccccccccccccccccc";

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "rustling-core-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn request(&self, hooks: Option<u32>) -> Vec<u8> {
        serde_json::to_vec(&json!({
            "command": "inspect_instance",
            "instanceId": "test:one",
            "gameRevision": REVISION,
            "nativeHooksApi": hooks,
            "instanceDir": self.0,
        }))
        .unwrap()
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn answer(request: &[u8]) -> Value {
    serde_json::from_str(&dispatch(request)).unwrap()
}

fn content(root: &Path, id: &str, extra_namespace: Option<&str>) {
    let manifest = format!(
        "id = \"{id}\"\nname = \"Test {id}\"\nversion = \"1.2.3\"\napi = \"0.2\"\nside = \"both\"\ncapabilities = [\"registries\"]\n{}",
        extra_namespace
            .map(|ns| format!("namespaces = [\"{ns}\"]\n"))
            .unwrap_or_default(),
    );
    let bytes = Package::write_files(&manifest, None, &[]).unwrap();
    let dir = root.join("horizon-mods");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!("{id}.hmod")), bytes).unwrap();
}

fn native(root: &Path, horizon: &str, hooks: Option<u32>) {
    use std::io::Write;
    let manifest = json!({
        "schema": 1, "id": "native_test", "version": "0.1.0", "game": "26.3",
        "horizon": horizon, "mixins": MIXINS, "hooks": hooks,
    });
    let mixin = format!(
        "[mod]\nid = \"native_test\"\nversion = \"0.1.0\"\n[compatibility]\nhorizon_revision = \"{horizon}\"\n"
    );
    let dir = root.join("horizon-native-mods");
    fs::create_dir_all(&dir).unwrap();
    let file = fs::File::create(dir.join("native_test.hnmod")).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();
    for (name, bytes) in [
        (
            "horizon-native.json",
            serde_json::to_vec(&manifest).unwrap(),
        ),
        ("mixins/native_test/horizon-mixin.toml", mixin.into_bytes()),
    ] {
        zip.start_file(name, options).unwrap();
        zip.write_all(&bytes).unwrap();
    }
    zip.finish().unwrap();
}

#[test]
fn ping_is_a_versioned_success_envelope() {
    let answer = answer(br#"{"command":"ping"}"#);
    assert_eq!(
        answer,
        json!({
            "schema": 1, "ok": true,
            "result": {"kind": "pong", "protocol": "rustling.launcher.ping.v1", "readOnly": true},
        })
    );
}

#[test]
fn bad_requests_are_data_and_large_requests_are_bounded() {
    let first = answer(b"{");
    assert_eq!(first["ok"], false);
    assert_eq!(first["error"]["code"], "invalid_request");
    let answer = answer(&vec![b' '; MAX_REQUEST_BYTES + 1]);
    assert_eq!(answer["error"]["code"], "invalid_request");
}

#[test]
fn reads_both_mod_kinds_and_the_real_switch_semantics() {
    let root = Scratch::new();
    content(&root.0, "alpha", None);
    native(&root.0, REVISION, Some(46));
    fs::write(root.0.join("horizon-mods.txt"), "mod.alpha:true\n").unwrap();
    let answer = answer(&root.request(Some(46)));
    assert_eq!(answer["ok"], true, "{answer}");
    let mods = answer["result"]["mods"].as_array().unwrap();
    assert_eq!(mods.len(), 2);
    let native_mod = mods.iter().find(|m| m["id"] == "native_test").unwrap();
    assert_eq!(native_mod["selected"], true);
    assert_eq!(native_mod["buildRequired"], true);
    assert_eq!(native_mod["problem"], Value::Null);
    let content_mod = mods.iter().find(|m| m["id"] == "alpha").unwrap();
    assert_eq!(content_mod["selected"], true);
    assert_eq!(content_mod["side"], "both");
    assert_eq!(content_mod["sha256"].as_str().unwrap().len(), 64);
    assert_eq!(
        hex(&Sha256::digest(b"abc")),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn invalid_mod_does_not_hide_a_valid_one() {
    let root = Scratch::new();
    content(&root.0, "good", None);
    fs::write(root.0.join("horizon-mods/bad.hmod"), b"not a zip").unwrap();
    let answer = answer(&root.request(None));
    let mods = answer["result"]["mods"].as_array().unwrap();
    assert_eq!(mods.len(), 2);
    assert_eq!(mods[0]["problem"]["code"], "mod_invalid");
    assert_eq!(mods[1]["id"], "good");
}

#[test]
fn native_revision_and_hook_contracts_follow_the_build_rules() {
    let root = Scratch::new();
    native(&root.0, OLD_REVISION, None);
    let mismatched = answer(&root.request(Some(46)));
    assert_eq!(
        mismatched["result"]["mods"][0]["problem"]["code"],
        "revision_mismatch"
    );

    native(&root.0, OLD_REVISION, Some(47));
    let unsupported = answer(&root.request(Some(46)));
    assert_eq!(
        unsupported["result"]["mods"][0]["problem"]["code"],
        "hook_api_unsupported"
    );
    let answer = answer(&root.request(Some(47)));
    assert_eq!(answer["result"]["mods"][0]["problem"], Value::Null);
}

#[test]
fn selected_content_namespace_clashes_are_named_on_the_later_file() {
    let root = Scratch::new();
    content(&root.0, "alpha", Some("shared"));
    content(&root.0, "beta", Some("shared"));
    fs::write(
        root.0.join("horizon-mods.txt"),
        "mod.alpha:true\nmod.beta:true\n",
    )
    .unwrap();
    let answer = answer(&root.request(None));
    let mods = answer["result"]["mods"].as_array().unwrap();
    assert_eq!(mods[0]["problem"], Value::Null);
    assert_eq!(mods[1]["problem"]["code"], "namespace_conflict");
    assert_eq!(mods[1]["problem"]["fields"][0]["value"], "shared");
}

#[test]
fn content_folder_is_not_hashed_as_a_file_and_scan_is_read_only() {
    let root = Scratch::new();
    let folder = root.0.join("horizon-mods/dev_mod");
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("horizon-mod.toml"), "id = \"dev_mod\"\nname = \"Dev\"\nversion = \"1\"\napi = \"0.2\"\ncapabilities = [\"registries\"]\n").unwrap();
    let answer = answer(&root.request(None));
    assert_eq!(answer["result"]["mods"][0]["sha256"], Value::Null);
    assert_eq!(answer["result"]["mods"][0]["problem"], Value::Null);
    assert!(!root.0.join("rustling-instance.json").exists());
}

#[test]
fn reads_the_real_native_current_pointer_and_build_metadata() {
    let root = Scratch::new();
    let dir = root.0.join("files/native/build-key");
    fs::create_dir_all(&dir).unwrap();
    fs::write(root.0.join("files/native/current"), "build-key\n").unwrap();
    fs::write(
        dir.join("build.json"),
        json!({"key": "build-key", "commit": REVISION}).to_string(),
    )
    .unwrap();
    let answer = answer(&root.request(None));
    assert_eq!(answer["result"]["nativeBuild"]["present"], true);
    assert_eq!(answer["result"]["nativeBuild"]["revisionMatches"], true);
    assert_eq!(answer["result"]["nativeBuild"]["buildKey"], "build-key");
}
