package dev.neurokitchen.quire.presentation.mapping

import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.toComposeImageBitmap
import dev.neurokitchen.quire.domain.model.PageImage
import org.jetbrains.skia.ColorAlphaType
import org.jetbrains.skia.ColorType
import org.jetbrains.skia.Image
import org.jetbrains.skia.ImageInfo
import java.io.File

actual fun PageImage.toImageBitmap(): ImageBitmap =
    Image.makeRaster(ImageInfo(width, height, ColorType.RGBA_8888, ColorAlphaType.OPAQUE), pixels, width * 4)
        .toComposeImageBitmap()

actual fun loadImageFile(path: String): ImageBitmap? =
    runCatching { Image.makeFromEncoded(File(path).readBytes()).toComposeImageBitmap() }.getOrNull()
