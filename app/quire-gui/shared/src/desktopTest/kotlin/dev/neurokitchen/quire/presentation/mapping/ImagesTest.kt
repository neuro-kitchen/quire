package dev.neurokitchen.quire.presentation.mapping

import java.io.File
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull

class ImagesTest {
    /** arXiv's HTML papers often have SVG figures; the raster decoder read none, so they did not show. */
    @Test
    fun svgFiguresAreDrawn() {
        val svg = File.createTempFile("figure", ".svg").apply {
            writeText("""<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300" viewBox="0 0 400 300"><rect x="10" y="10" width="380" height="280" fill="none" stroke="black"/><circle cx="200" cy="150" r="80" fill="#3b4fd8"/></svg>""")
            deleteOnExit()
        }
        val image = assertNotNull(loadImageFile(svg.path))
        assertEquals(4f / 3f, image.width.toFloat() / image.height, 0.01f)
    }

    @Test
    fun rasterFiguresStillWorkAndJunkIsNone() {
        val png = File.createTempFile("figure", ".png").apply {
            writeBytes(org.jetbrains.skia.Surface.makeRasterN32Premul(20, 10).makeImageSnapshot().encodeToData()!!.bytes)
            deleteOnExit()
        }
        assertEquals(20, assertNotNull(loadImageFile(png.path)).width)
        val junk = File.createTempFile("figure", ".png").apply { writeText("not an image"); deleteOnExit() }
        assertNull(loadImageFile(junk.path))
    }
}
