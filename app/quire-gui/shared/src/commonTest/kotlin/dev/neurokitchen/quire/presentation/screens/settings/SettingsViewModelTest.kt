package dev.neurokitchen.quire.presentation.screens.settings

import dev.neurokitchen.quire.data.repository.SampleLibraryRepository
import dev.neurokitchen.quire.data.repository.SampleSettingsRepository
import dev.neurokitchen.quire.domain.model.SettingEntry
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class SettingsViewModelTest {
    private val dispatcher = StandardTestDispatcher()

    @BeforeTest
    fun setUp() = Dispatchers.setMain(dispatcher)

    @AfterTest
    fun tearDown() = Dispatchers.resetMain()

    private val settings = SampleSettingsRepository(
        entries = listOf(SettingEntry("user.name", "Ada"), SettingEntry("export.thesis.format", "biblatex")),
    )

    private fun TestScope.viewModel() = SettingsViewModel(settings, SampleLibraryRepository()).also { advanceUntilIdle() }

    @Test
    fun sectionsShowTheirKeysWithSavedValues() = runTest(dispatcher) {
        val vm = viewModel()
        val general = vm.state.value.fields
        assertEquals(listOf("user.name", "user.email"), general.map { it.key })
        assertEquals("Ada", general[0].saved)
        assertNull(general[1].saved)

        vm.onEvent(SettingsEvent.OnSectionClick(SettingsSection.Export))
        // `export.*.format` shows one row per profile in the file.
        assertEquals(listOf("export.thesis.format"), vm.state.value.fields.map { it.key })
    }

    @Test
    fun typedValuesApplyOnRequestChoicesAtOnce() = runTest(dispatcher) {
        val vm = viewModel()
        vm.onEvent(SettingsEvent.OnDraftChange("user.email", "ada@example.org"))
        assertTrue(vm.state.value.fields.first { it.key == "user.email" }.isDirty)
        vm.onEvent(SettingsEvent.OnApply("user.email"))
        advanceUntilIdle()
        assertEquals("ada@example.org", settings.entries().first { it.key == "user.email" }.value)
        assertFalse(vm.state.value.fields.first { it.key == "user.email" }.isDirty)

        vm.onEvent(SettingsEvent.OnSectionClick(SettingsSection.Reader))
        vm.onEvent(SettingsEvent.OnDraftChange("reader.theme", "sepia"))
        advanceUntilIdle()
        assertEquals("sepia", settings.entries().first { it.key == "reader.theme" }.value)

        vm.onEvent(SettingsEvent.OnReset("reader.theme"))
        advanceUntilIdle()
        assertTrue(settings.entries().none { it.key == "reader.theme" })
    }

    @Test
    fun theRawFileIsSavedAndReverted() = runTest(dispatcher) {
        val vm = viewModel()
        vm.onEvent(SettingsEvent.OnRawChange("[user]\nname = \"Ada\"\n"))
        assertTrue(vm.state.value.raw.isDirty)
        vm.onEvent(SettingsEvent.OnRawSave)
        advanceUntilIdle()
        assertEquals("[user]\nname = \"Ada\"\n", settings.readFile())
        assertFalse(vm.state.value.raw.isDirty)

        vm.onEvent(SettingsEvent.OnRawChange("oops"))
        vm.onEvent(SettingsEvent.OnRawRevert)
        assertEquals("[user]\nname = \"Ada\"\n", vm.state.value.raw.text)
    }
}
