package dev.neurokitchen.quire.presentation.mapping

import android.graphics.Bitmap
import android.graphics.BitmapFactory
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import dev.neurokitchen.quire.domain.model.PageImage
import java.nio.ByteBuffer

// ARGB_8888 is stored as R, G, B, A bytes: the same order as the engine's pixels.
actual fun PageImage.toImageBitmap(): ImageBitmap =
    Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
        .apply { copyPixelsFromBuffer(ByteBuffer.wrap(pixels)) }
        .asImageBitmap()

actual fun loadImageFile(path: String): ImageBitmap? = BitmapFactory.decodeFile(path)?.asImageBitmap()
