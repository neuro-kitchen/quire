//! Android only: the app hands the engine its JVM context once, at start, before any network call.
//! reqwest checks TLS certificates with `rustls-platform-verifier`, which on Android asks the
//! system's verifier (its trust store, user-installed certificates included) through the JVM.

use jni::EnvUnowned;
use jni::errors::ThrowRuntimeExAndDefault;
use jni::jni_mangle;
use jni::objects::{JClass, JObject};

/// `AndroidTls.init(context)` in the app (`dev.neurokitchen.quire.data.engine.AndroidTls`).
#[jni_mangle("dev.neurokitchen.quire.data.engine.AndroidTls")]
pub fn init<'caller>(mut unowned_env: EnvUnowned<'caller>, _class: JClass<'caller>, context: JObject<'caller>) {
    unowned_env
        .with_env(|env| rustls_platform_verifier::android::init_with_env(env, context))
        .resolve::<ThrowRuntimeExAndDefault>();
}
