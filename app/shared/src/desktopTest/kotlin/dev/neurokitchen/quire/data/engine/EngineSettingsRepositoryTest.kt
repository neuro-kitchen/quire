package dev.neurokitchen.quire.data.engine

import kotlinx.coroutines.runBlocking
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

/** The settings calls through the generated bindings, on a throwaway config file. */
class EngineSettingsRepositoryTest {
    private val file = Files.createTempDirectory("quire-config").resolve("config.toml")
    private val settings = EngineSettingsRepository(file.toString())

    @Test
    fun valuesAreCheckedAndWritten() = runBlocking {
        assertTrue(settings.keys().any { it.pattern == "reader.width" })
        settings.set("reader.width", "80")
        assertEquals("80", settings.entries().first { it.key == "reader.width" }.value)
        val refused = assertFailsWith<Exception> { settings.set("reader.width", "5") }
        assertTrue("40 to 120" in refused.message.orEmpty(), refused.message)
        assertEquals(listOf("unknown key `colour.bad`"), settings.check("[colour]\nbad = 1\n"))
        assertFailsWith<Exception> { settings.writeFile("[broken") }
        assertTrue("width = 80" in settings.readFile())
    }
}
