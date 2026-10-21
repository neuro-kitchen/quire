import java.util.Properties
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.androidApplication)
    alias(libs.plugins.composeCompiler)
}

kotlin {
    compilerOptions {
        jvmTarget = JvmTarget.JVM_17
    }
}

// ── The Rust engine for Android ───────────────────────────────────────────────────────────────
// cargo-ndk builds quire-ffi for each ABI into build/generated/jniLibs/<abi>/libquire_ffi.so,
// packaged with the app. Needs the NDK (`quire.ndk.dir` in local.properties, or ANDROID_NDK_HOME; not `ndk.dir`, which the
// Android plugin reads for its own C++ builds), the
// Rust targets (`rustup target add aarch64-linux-android x86_64-linux-android`) and cargo-ndk.
// `-Pquire.abis=x86_64` builds the emulator's only (faster); `-Pquire.release` optimises.
val repoRoot = rootProject.layout.projectDirectory.dir("../..")
val rustProfile = if (providers.gradleProperty("quire.release").isPresent) "app-release" else "app-dev"
val abis = providers.gradleProperty("quire.abis").getOrElse("arm64-v8a,x86_64").split(',').map { it.trim() }
val jniLibsDir = layout.buildDirectory.dir("generated/jniLibs")
val ndkDir: String? = System.getenv("ANDROID_NDK_HOME") ?: rootProject.file("local.properties").takeIf { it.exists() }
    ?.let { file -> Properties().apply { file.inputStream().use { load(it) } }.getProperty("quire.ndk.dir") }

val cargoNdkBuildFfi by tasks.registering(Exec::class) {
    description = "Builds the engine (quire-ffi) for Android's ABIs with cargo-ndk."
    workingDir = repoRoot.asFile
    ndkDir?.let { environment("ANDROID_NDK_HOME", it) }
    commandLine(
        listOf("cargo", "ndk") + abis.flatMap { listOf("-t", it) } +
            listOf("-P", "31", "-o", jniLibsDir.get().asFile.path, "build", "-p", "quire-ffi", "--profile", rustProfile),
    )
    // Cargo decides what is up to date; Gradle always asks it.
    outputs.upToDateWhen { false }
    // cargo-ndk copies every shared library cargo made (iroh and pdf_inspector build one too);
    // quire_ffi links them statically, so only it ships.
    // A local (a script-level value would tie the task to the script: no configuration cache).
    val out = jniLibsDir.get().asFile
    doLast {
        out.walkTopDown().filter { it.isFile && it.name != "libquire_ffi.so" }.forEach { it.delete() }
    }
}

android {
    sourceSets.getByName("main").jniLibs.directories.add(jniLibsDir.get().asFile.path)
    namespace = "dev.neurokitchen.quire"
    compileSdk = libs.versions.android.compileSdk.get().toInt()

    defaultConfig {
        applicationId = "dev.neurokitchen.quire"
        minSdk = libs.versions.android.minSdk.get().toInt()
        targetSdk = libs.versions.android.targetSdk.get().toInt()
        versionCode = 1
        versionName = "0.1.0"
        // Only the ABIs the engine is built for (JNA brings more, e.g. mips).
        ndk { abiFilters += abis }
    }
    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    buildFeatures {
        compose = true
    }
}

dependencies {
    implementation(project(":shared"))
    implementation(libs.androidx.activity.compose)
    implementation(libs.koin.android)
    implementation(libs.compose.uiToolingPreview)
    debugImplementation(libs.compose.uiTooling)
}

tasks.matching { it.name.startsWith("merge") && it.name.endsWith("JniLibFolders") }.configureEach { dependsOn(cargoNdkBuildFfi) }
