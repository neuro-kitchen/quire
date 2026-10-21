package dev.neurokitchen.quire.presentation.platform

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.add_file_pick_bib
import dev.neurokitchen.quire.resources.add_file_pick_zotero
import dev.neurokitchen.quire.resources.add_pdf_pick_title
import org.jetbrains.compose.resources.stringResource

/** The desktop's dialogs are modal: the path comes back at once. */
@Composable
actual fun rememberFilePicker(onPicked: (PickKind, List<String>) -> Unit): FilePicker {
    val callback by rememberUpdatedState(onPicked)
    val pdfTitle = stringResource(Res.string.add_pdf_pick_title)
    val bibTitle = stringResource(Res.string.add_file_pick_bib)
    val folderTitle = stringResource(Res.string.add_file_pick_zotero)
    return remember {
        object : FilePicker {
            override val canPickFolders = true
            override fun pick(kind: PickKind) {
                val path = when (kind) {
                    PickKind.Pdfs -> pickFile(pdfTitle, listOf("pdf"))
                    PickKind.Bib -> pickFile(bibTitle, listOf("bib"))
                    PickKind.Folder -> pickFolder(folderTitle)
                }
                if (path != null) callback(kind, listOf(path))
            }
        }
    }
}
