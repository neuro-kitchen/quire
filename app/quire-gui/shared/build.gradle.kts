import java.net.URI
import java.security.MessageDigest
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.androidMultiplatformLibrary)
    alias(libs.plugins.composeMultiplatform)
    alias(libs.plugins.composeCompiler)
    alias(libs.plugins.kotlinSerialization)
}

// ── The Rust engine ────────────────────────────────────────────────────────────────────────────
// `cargo build -p quire-ffi` makes the native library; `uniffi-bindgen` generates its Kotlin API
// from it. The library goes into the desktop jar under JNA's platform folder (linux-x86-64/…),
// where JNA finds it. `-Pquire.release` builds the engine optimised (for packaging).
val repoRoot = rootProject.layout.projectDirectory.dir("../..")
// Cargo profile and its folder under target/ (`app-dev`: no debug info, see Cargo.toml).
val rustProfile = if (providers.gradleProperty("quire.release").isPresent) "app-release" else "app-dev"
val uniffiKotlinDir = layout.buildDirectory.dir("generated/uniffi/kotlin")
val nativeResourcesDir = layout.buildDirectory.dir("generated/uniffi/resources")
val hostOs = System.getProperty("os.name").lowercase()
val hostArch = System.getProperty("os.arch").let { if (it == "amd64" || it == "x86_64") "x86-64" else if (it == "aarch64") "aarch64" else it }
val (jnaOs, libraryFile) = when {
    hostOs.contains("mac") -> "darwin" to "libquire_ffi.dylib"
    hostOs.contains("windows") -> "win32" to "quire_ffi.dll"
    else -> "linux" to "libquire_ffi.so"
}
val nativeLibrary = repoRoot.file("target/$rustProfile/$libraryFile")

val cargoBuildFfi by tasks.registering(Exec::class) {
    description = "Builds the engine's native library (quire-ffi)."
    workingDir = repoRoot.asFile
    commandLine("cargo", "build", "-p", "quire-ffi", "--profile", rustProfile)
    // Cargo decides what is up to date; Gradle always asks it.
    outputs.upToDateWhen { false }
}

val generateUniffiKotlin by tasks.registering(Exec::class) {
    description = "Generates the engine's Kotlin API from the native library."
    dependsOn(cargoBuildFfi)
    workingDir = repoRoot.asFile
    inputs.file(nativeLibrary)
    outputs.dir(uniffiKotlinDir)
    commandLine(
        "cargo", "run", "-q", "-p", "uniffi-bindgen", "--", "generate",
        "--library", nativeLibrary.asFile.path, "--language", "kotlin", "--no-format",
        "--out-dir", uniffiKotlinDir.get().asFile.path,
    )
}

val copyNativeLibrary by tasks.registering(Copy::class) {
    description = "Puts the native library where JNA looks for it on the classpath."
    dependsOn(cargoBuildFfi)
    from(nativeLibrary)
    into(nativeResourcesDir.map { it.dir("$jnaOs-$hostArch") })
}

tasks.matching { it.name == "compileKotlinDesktop" }.configureEach { dependsOn(generateUniffiKotlin) }
tasks.matching { it.name == "desktopProcessResources" }.configureEach { dependsOn(copyNativeLibrary) }
// Tests open throwaway libraries: no system keyring.
tasks.matching { it.name == "desktopTest" }.configureEach {
    this as Test
    environment("QUIRE_NO_KEYRING", "1")
}

kotlin {
    android {
        namespace = "dev.neurokitchen.quire.shared"
        compileSdk = libs.versions.android.compileSdk.get().toInt()
        minSdk = libs.versions.android.minSdk.get().toInt()
        compilerOptions {
            jvmTarget = JvmTarget.JVM_17
        }
        androidResources {
            enable = true
        }
        withHostTest {}
    }

    jvm("desktop") {
        compilerOptions {
            jvmTarget = JvmTarget.JVM_17
        }
    }

    // Compiled only on macOS; on Linux Gradle skips them with a warning.
    listOf(iosArm64(), iosSimulatorArm64()).forEach { target ->
        target.binaries.framework {
            baseName = "Shared"
            isStatic = true
        }
    }

    sourceSets {
        commonMain.dependencies {
            implementation(libs.compose.runtime)
            implementation(libs.compose.foundation)
            implementation(libs.compose.ui)
            implementation(libs.compose.material3)
            implementation(libs.compose.material3.navigationSuite)
            implementation(libs.compose.components.resources)
            implementation(libs.compose.uiToolingPreview)
            implementation(libs.androidx.lifecycle.viewmodelCompose)
            implementation(libs.androidx.lifecycle.runtimeCompose)
            implementation(libs.kotlinx.coroutines.core)
            api(libs.koin.core)
            implementation(libs.koin.compose.viewmodel)
            implementation(libs.navigation3.ui)
            implementation(libs.lifecycle.viewmodel.navigation3)
            implementation(libs.kotlinx.serialization.core)
            implementation(libs.haze)
            implementation(libs.haze.blur)
            implementation(libs.haze.blur.materials)
        }
        commonTest.dependencies {
            implementation(libs.kotlin.test)
            implementation(libs.kotlinx.coroutines.test)
            implementation(libs.koin.test)
        }
        androidMain.dependencies {
            implementation(libs.compose.uiTooling)
        }
        getByName("desktopMain") {
            // The engine's Kotlin API, generated from quire-ffi (see the Rust engine section below).
            kotlin.srcDir(uniffiKotlinDir)
            resources.srcDir(nativeResourcesDir)
            dependencies {
                implementation(libs.kotlinx.coroutines.swing)
                implementation(libs.jna)
            }
        }
        getByName("desktopTest").dependencies {
            implementation(compose.desktop.currentOs)
        }
    }
}

compose.resources {
    packageOfResClass = "dev.neurokitchen.quire.resources"
    // The desktop and Android shells read strings too (menu bar, window titles).
    publicResClass = true
}

dependencies {
    androidRuntimeClasspath(libs.compose.uiTooling)
}
