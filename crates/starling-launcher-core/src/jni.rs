//! The only Android-specific adapter. Panics become protocol errors, never cross into the JVM.
use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::jstring;

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_juanluci_starling_RustlingCore_dispatch(
    mut env: JNIEnv,
    _class: JClass,
    request: JString,
) -> jstring {
    let response = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let request = env.get_string(&request).map(String::from)?;
        Ok::<_, jni::errors::Error>(crate::dispatch(request.as_bytes()))
    }));
    let text = match response {
        Ok(Ok(text)) => text,
        Ok(Err(error)) => error_json("invalid_request", &error.to_string()),
        Err(_) => error_json("internal_error", "launcher worker failed"),
    };
    env.new_string(text)
        .map_or(std::ptr::null_mut(), |text| text.into_raw())
}

fn error_json(code: &str, detail: &str) -> String {
    serde_json::json!({
        "schema": crate::SCHEMA,
        "ok": false,
        "error": { "code": code, "detail": detail, "fields": [] },
    })
    .to_string()
}
