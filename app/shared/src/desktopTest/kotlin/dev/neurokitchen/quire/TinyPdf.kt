package dev.neurokitchen.quire

/** A one-page US Letter PDF with `lines` of Helvetica text from the top (as in the Rust tests). */
fun tinyPdf(lines: List<String>): ByteArray {
    val text = lines.mapIndexed { i, line -> "BT /F1 12 Tf 72 ${720 - i * 18} Td ($line) Tj ET" }.joinToString("\n")
    val objects = listOf(
        "<< /Type /Catalog /Pages 2 0 R >>",
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>",
        "<< /Length ${text.length} >>\nstream\n$text\nendstream",
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
    )
    val pdf = StringBuilder("%PDF-1.4\n")
    val offsets = objects.mapIndexed { i, body ->
        pdf.length.also { pdf.append("${i + 1} 0 obj\n$body\nendobj\n") }
    }
    val xref = pdf.length
    pdf.append("xref\n0 ${objects.size + 1}\n0000000000 65535 f \n")
    offsets.forEach { pdf.append("%010d 00000 n \n".format(it)) }
    pdf.append("trailer\n<< /Size ${objects.size + 1} /Root 1 0 R >>\nstartxref\n$xref\n%%EOF\n")
    return pdf.toString().toByteArray()
}
