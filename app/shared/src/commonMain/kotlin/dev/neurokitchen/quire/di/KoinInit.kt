package dev.neurokitchen.quire.di

import org.koin.core.KoinApplication
import org.koin.core.context.startKoin
import org.koin.core.module.Module
import org.koin.dsl.KoinAppDeclaration

/** Starts dependency injection once per process; each platform's entry point calls it. */
fun initKoin(data: Module = sampleDataModule, platform: KoinAppDeclaration = {}): KoinApplication = startKoin {
    platform()
    modules(appModules(data))
}
