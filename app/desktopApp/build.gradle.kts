import org.jetbrains.compose.desktop.application.dsl.TargetFormat
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.kotlinJvm)
    alias(libs.plugins.composeMultiplatform)
    alias(libs.plugins.composeCompiler)
}

kotlin {
    compilerOptions {
        jvmTarget = JvmTarget.JVM_17
    }
}

java {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
}

dependencies {
    implementation(project(":shared"))
    implementation(compose.desktop.currentOs)
    implementation(libs.compose.components.resources)
    implementation(libs.kotlinx.coroutines.swing)
}

compose.desktop {
    application {
        mainClass = "dev.neurokitchen.quire.MainKt"
        nativeDistributions {
            // PDFium for the reader (downloaded and checked by :shared:extractPdfium).
            appResourcesRootDir.set(project(":shared").layout.buildDirectory.dir("pdfium/resources"))
            targetFormats(TargetFormat.Dmg, TargetFormat.Msi, TargetFormat.Deb, TargetFormat.Rpm)
            packageName = "quire"
            packageVersion = "0.1.0"
        }
    }
}

tasks.matching { it.name == "prepareAppResources" }.configureEach { dependsOn(":shared:extractPdfium") }
