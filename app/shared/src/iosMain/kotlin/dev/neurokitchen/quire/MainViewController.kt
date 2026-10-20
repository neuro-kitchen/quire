package dev.neurokitchen.quire

import androidx.compose.ui.window.ComposeUIViewController
import dev.neurokitchen.quire.di.initKoin

/** Called once from Swift at launch. */
fun startKoin() {
    initKoin()
}

fun MainViewController() = ComposeUIViewController { App() }
