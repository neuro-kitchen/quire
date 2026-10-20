package dev.neurokitchen.quire.presentation.mapping

import androidx.compose.ui.graphics.ImageBitmap
import dev.neurokitchen.quire.domain.model.PageImage

/** A rendered PDF page as an image Compose draws (RGBA pixels, opaque). */
expect fun PageImage.toImageBitmap(): ImageBitmap

/** An image file (PNG, JPEG…) as an image Compose draws; `null` if it cannot be read. */
expect fun loadImageFile(path: String): ImageBitmap?
