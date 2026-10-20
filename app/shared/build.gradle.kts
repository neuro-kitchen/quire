import java.net.URI
import java.security.MessageDigest
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.androidMultiplatformLibrary)
    alias(libs.plugins.composeMultiplatform)
    alias(libs.plugins.composeCompiler)
}

// ── The Rust engine ────────────────────────────────────────────────────────────────────────────
// `cargo build -p quire-ffi` makes the native library; `uniffi-bindgen` generates its Kotlin API
// from it. The library goes into the desktop jar under JNA's platform folder (linux-x86-64/…),
// where JNA finds it. `-Pquire.release` builds the engine optimised (for packaging).
val repoRoot = rootProject.layout.projectDirectory.dir("..")
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

// ── PDFium (the reader's PDF pages) ────────────────────────────────────────────────────────────
// Prebuilt by bblanchon/pdfium-binaries (MIT; PDFium itself BSD-3/Apache-2.0). The version must
// match the API `pdfium-render` binds (`pdfium_7881` in Cargo.toml). Checksums pin each download.
val pdfiumVersion = "7881"
val pdfiumBuilds = mapOf(
    "linux-x64" to "1470e21b8b4a3b4ad7f85684e2da11d94f3b69a86d81dee11b9b6709d927ac1d",
    "mac-arm64" to "52e94ca5aa8847934330daf3f8150c190682c5ca93831468794f8b90d4392e40",
    "mac-x64" to "6dedf83990e0e3d6b7c93c9e7589c5a126b0ae14b7464d76120cff7a26afb18b",
    "win-x64" to "73cc0de638ac2095e7445bf56a38200a5b7c7ca0e9f4ba144598f2457377ac08",
)
val pdfiumHost = when {
    hostOs.contains("mac") -> if (hostArch == "aarch64") "mac-arm64" else "mac-x64"
    hostOs.contains("windows") -> "win-x64"
    else -> "linux-x64"
}
// Compose's names for app resource folders (`appResourcesRootDir`).
val composeHost = mapOf("linux-x64" to "linux-x64", "mac-arm64" to "macos-arm64", "mac-x64" to "macos-x64", "win-x64" to "windows-x64").getValue(pdfiumHost)
val pdfiumArchive = layout.buildDirectory.file("pdfium/pdfium-$pdfiumVersion-$pdfiumHost.tgz")
/** App resources root: `<compose host>/` holds the PDFium library (read by the desktop app). */
val pdfiumResources = layout.buildDirectory.dir("pdfium/resources")

val downloadPdfium by tasks.registering {
    description = "Downloads PDFium $pdfiumVersion for this computer and checks its SHA-256."
    val url = "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/$pdfiumVersion/pdfium-$pdfiumHost.tgz"
    val expected = pdfiumBuilds.getValue(pdfiumHost)
    val archive = pdfiumArchive
    outputs.file(archive)
    doLast {
        val file = archive.get().asFile
        file.parentFile.mkdirs()
        URI(url).toURL().openStream().use { input -> file.outputStream().use { input.copyTo(it) } }
        val digest = MessageDigest.getInstance("SHA-256").digest(file.readBytes())
        val actual = digest.joinToString("") { "%02x".format(it) }
        if (actual != expected) {
            file.delete()
            throw GradleException("PDFium download $url has SHA-256 $actual, expected $expected")
        }
    }
}

val extractPdfium by tasks.registering(Copy::class) {
    description = "Puts the PDFium library where the app and the tests find it."
    dependsOn(downloadPdfium)
    from(tarTree(pdfiumArchive)) {
        include("lib/*pdfium*.so", "lib/*pdfium*.dylib", "bin/pdfium.dll", "LICENSE", "licenses/**")
        eachFile { if (path.startsWith("lib/") || path.startsWith("bin/")) path = name }
        includeEmptyDirs = false
    }
    into(pdfiumResources.map { it.dir(composeHost) })
}

tasks.matching { it.name == "compileKotlinDesktop" }.configureEach { dependsOn(generateUniffiKotlin) }
tasks.matching { it.name == "desktopProcessResources" }.configureEach { dependsOn(copyNativeLibrary) }
// Tests open throwaway libraries: no system keyring.
tasks.matching { it.name == "desktopTest" }.configureEach {
    this as Test
    dependsOn(extractPdfium)
    environment("QUIRE_NO_KEYRING", "1")
    systemProperty("quire.pdfium.dir", pdfiumResources.get().dir(composeHost).asFile.path)
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
