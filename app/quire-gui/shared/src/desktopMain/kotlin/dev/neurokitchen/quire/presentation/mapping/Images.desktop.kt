package dev.neurokitchen.quire.presentation.mapping

import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.toComposeImageBitmap
import dev.neurokitchen.quire.domain.model.PageImage
import org.jetbrains.skia.ColorAlphaType
import org.jetbrains.skia.ColorType
import org.jetbrains.skia.Data
import org.jetbrains.skia.Image
import org.jetbrains.skia.ImageInfo
import org.jetbrains.skia.Surface
import org.jetbrains.skia.svg.SVGDOM
import java.io.File

actual fun PageImage.toImageBitmap(): ImageBitmap =
    Image.makeRaster(ImageInfo(width, height, ColorType.RGBA_8888, ColorAlphaType.OPAQUE), pixels, width * 4)
        .toComposeImageBitmap()

/** A figure file: PNG, JPEG, WebP or GIF decoded; SVG (arXiv's HTML papers) drawn. */
actual fun loadImageFile(path: String): ImageBitmap? = runCatching {
    val bytes = File(path).readBytes()
    if (isSvg(path, bytes)) drawSvg(bytes) else Image.makeFromEncoded(bytes).toComposeImageBitmap()
}.getOrNull()

private fun isSvg(path: String, bytes: ByteArray): Boolean =
    path.endsWith(".svg", ignoreCase = true) ||
        bytes.copyOfRange(0, minOf(bytes.size, 512)).decodeToString().let { "<svg" in it }

/** Widest an SVG figure is drawn, in pixels. */
private const val SVG_WIDTH = 1600

/** The SVG at its own aspect ratio (its size, else its view box), [SVG_WIDTH] pixels wide. */
private fun drawSvg(bytes: ByteArray): ImageBitmap? {
    val dom = SVGDOM(Data.makeFromBytes(bytes))
    val root = dom.root ?: return null
    val box = root.viewBox
    val (w, h) = when {
        root.width.value > 0f && root.height.value > 0f -> root.width.value to root.height.value
        box != null && box.width > 0f && box.height > 0f -> box.width to box.height
        else -> 800f to 600f
    }
    val width = SVG_WIDTH
    val height = (width * h / w).toInt().coerceAtLeast(1)
    dom.setContainerSize(w, h)
    val surface = Surface.makeRasterN32Premul(width, height)
    surface.canvas.clear(org.jetbrains.skia.Color.WHITE)
    surface.canvas.scale(width / w, height / h)
    dom.render(surface.canvas)
    return surface.makeImageSnapshot().toComposeImageBitmap()
}
