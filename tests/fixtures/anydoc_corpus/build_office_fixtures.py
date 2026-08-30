#!/usr/bin/env python3
"""Hand-build minimal, hand-verifiable fixtures for the S6 twelve-format wave.

Sibling of `build_docx_fixtures.py`, same discipline and for the same reason:
no external dependency, every part written here as a literal string, so a
reviewer can unzip (or `cat`) a fixture, read the one text-bearing part in a
text editor, and know the exact answer the extractor owes -- before any
extractor runs. `MANIFEST.md`'s "Office fixture corpus (S6)" section records
that hand-known answer per fixture; `office.rs`'s own tests assert it.

Every format here is either a ZIP of XML parts (ODF, OOXML, EPUB) or a plain
text format (RTF, PDF), which is exactly why they are hand-authorable at all.
The two binary OLE formats anydoc also parses -- Word 97 `.doc` and
PowerPoint 97 `.ppt` -- are NOT hand-authorable this way (an OLE2 compound
file plus BIFF/PPT record streams is not something a reviewer can read in a
text editor either), and are recorded as a named corpus gap in MANIFEST.md
rather than faked. `08-doc-rtf-in-disguise.doc` covers the one `.doc` path
that IS hand-authorable, and is a real-world case anydoc documents by name.

Run: python3 build_office_fixtures.py <out_dir>
"""
import os
import sys
import zipfile

# --------------------------------------------------------------------- RTF

# Two paragraphs (\par separated) and nothing else -- RTF is a plain text
# format, so this fixture IS its own source listing.
RTF_PLAIN = (
    "{\\rtf1\\ansi\\deff0{\\fonttbl{\\f0 Times New Roman;}}\n"
    "\\pard First rtf paragraph.\\par\n"
    "\\pard Second rtf paragraph.\\par\n"
    "}\n"
)

# Adversarial: 400 nested groups before any text. anydoc's package limits cap
# nesting depth; this is the RTF-shaped equivalent of the docx corpus's
# hostile fixture -- a bounded-input refusal, not a parse.
RTF_DEEP_NESTING = (
    "{\\rtf1\\ansi\\deff0" + ("{" * 400) + "deep" + ("}" * 400) + "}"
)

# ------------------------------------------------------------------ ODF

ODF_NS = (
    'xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" '
    'xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" '
    'xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0" '
    'xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0" '
    'xmlns:presentation="urn:oasis:names:tc:opendocument:xmlns:presentation:1.0" '
    'xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"'
)

ODT_CONTENT = f"""<?xml version="1.0" encoding="UTF-8"?>
<office:document-content {ODF_NS}>
  <office:body>
    <office:text>
      <text:h text:outline-level="1">Odt Introduction</text:h>
      <text:p>First odt paragraph.</text:p>
      <text:h text:outline-level="2">Odt Background</text:h>
      <text:p>Second odt paragraph.</text:p>
    </office:text>
  </office:body>
</office:document-content>
"""

ODS_CONTENT = f"""<?xml version="1.0" encoding="UTF-8"?>
<office:document-content {ODF_NS}>
  <office:body>
    <office:spreadsheet>
      <table:table table:name="Budget">
        <table:table-row>
          <table:table-cell office:value-type="string"><text:p>Item</text:p></table:table-cell>
          <table:table-cell office:value-type="string"><text:p>Cost</text:p></table:table-cell>
        </table:table-row>
        <table:table-row>
          <table:table-cell office:value-type="string"><text:p>Widget</text:p></table:table-cell>
          <table:table-cell office:value-type="string"><text:p>10</text:p></table:table-cell>
        </table:table-row>
        <table:table-row>
          <table:table-cell office:value-type="string"><text:p>Gadget</text:p></table:table-cell>
          <table:table-cell office:value-type="string"><text:p>20</text:p></table:table-cell>
        </table:table-row>
      </table:table>
    </office:spreadsheet>
  </office:body>
</office:document-content>
"""

ODP_CONTENT = f"""<?xml version="1.0" encoding="UTF-8"?>
<office:document-content {ODF_NS}>
  <office:body>
    <office:presentation>
      <draw:page draw:name="page1">
        <draw:frame presentation:class="title">
          <draw:text-box><text:p>Odp Slide One</text:p></draw:text-box>
        </draw:frame>
        <draw:frame presentation:class="outline">
          <draw:text-box><text:p>Odp first bullet.</text:p></draw:text-box>
        </draw:frame>
      </draw:page>
      <draw:page draw:name="page2">
        <draw:frame presentation:class="title">
          <draw:text-box><text:p>Odp Slide Two</text:p></draw:text-box>
        </draw:frame>
        <draw:frame presentation:class="outline">
          <draw:text-box><text:p>Odp second bullet.</text:p></draw:text-box>
        </draw:frame>
      </draw:page>
    </office:presentation>
  </office:body>
</office:document-content>
"""

# Deliberately malformed: `<text:p>` is never closed. A well-formed zip and a
# well-formed manifest, one unreadable part -- the ODF-shaped twin of the
# docx corpus's `05-malformed-unclosed-element.docx`.
ODT_MALFORMED = f"""<?xml version="1.0" encoding="UTF-8"?>
<office:document-content {ODF_NS}>
  <office:body>
    <office:text>
      <text:h text:outline-level="1">Odt Introduction</text:h>
      <text:p>An unclosed paragraph.
    </office:text>
  </office:body>
</office:document-content>
"""

# Encrypted ODF: the package manifest carries `manifest:encryption-data` on a
# file entry, which is exactly what anydoc's `odf::is_encrypted` reads (and
# it parses the manifest properly rather than substring-matching, so this has
# to be a real element, not a comment).
ODF_ENCRYPTED_MANIFEST = """<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0">
  <manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml">
    <manifest:encryption-data manifest:checksum-type="SHA1/1K" manifest:checksum="AAAA"/>
  </manifest:file-entry>
</manifest:manifest>
"""

ODF_PLAIN_MANIFEST = """<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0">
  <manifest:file-entry manifest:full-path="/" manifest:media-type="{mime}"/>
  <manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
</manifest:manifest>
"""

ODF_MIME = {
    "odt": "application/vnd.oasis.opendocument.text",
    "ods": "application/vnd.oasis.opendocument.spreadsheet",
    "odp": "application/vnd.oasis.opendocument.presentation",
}


def write_odf(path, kind, content, manifest=None):
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        # `mimetype` first and STORED is the ODF packaging convention.
        z.writestr(zipfile.ZipInfo("mimetype"), ODF_MIME[kind], zipfile.ZIP_STORED)
        z.writestr(
            "META-INF/manifest.xml",
            manifest or ODF_PLAIN_MANIFEST.format(mime=ODF_MIME[kind]),
        )
        z.writestr("content.xml", content)


# ----------------------------------------------------------------- OOXML

PKG_RELS = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="{target}"/>
</Relationships>
"""

PRESENTATION_XML = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
                xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:sldIdLst>
    <p:sldId id="256" r:id="rId1"/>
    <p:sldId id="257" r:id="rId2"/>
  </p:sldIdLst>
</p:presentation>
"""

PRESENTATION_RELS = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide2.xml"/>
</Relationships>
"""

SLIDE_XML = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
       xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:cSld><p:spTree>
    <p:sp>
      <p:nvSpPr><p:nvPr><p:ph type="title"/></p:nvPr></p:nvSpPr>
      <p:txBody><a:p><a:r><a:t>{title}</a:t></a:r></a:p></p:txBody>
    </p:sp>
    <p:sp>
      <p:nvSpPr><p:nvPr><p:ph type="body" idx="1"/></p:nvPr></p:nvSpPr>
      <p:txBody><a:p><a:r><a:t>{body}</a:t></a:r></a:p></p:txBody>
    </p:sp>
  </p:spTree></p:cSld>
</p:sld>
"""

WORKBOOK_XML = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"
          xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets><sheet name="Budget" sheetId="1" r:id="rId1"/></sheets>
</workbook>
"""

WORKBOOK_RELS = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
</Relationships>
"""

SHEET_XML = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>
    <row r="1">
      <c r="A1" t="inlineStr"><is><t>Item</t></is></c>
      <c r="B1" t="inlineStr"><is><t>Cost</t></is></c>
    </row>
    <row r="2">
      <c r="A2" t="inlineStr"><is><t>Widget</t></is></c>
      <c r="B2"><v>10</v></c>
    </row>
    <row r="3">
      <c r="A3" t="inlineStr"><is><t>Gadget</t></is></c>
      <c r="B3"><v>20</v></c>
    </row>
  </sheetData>
</worksheet>
"""

# Deliberately malformed: `<a:t>` is never closed in slide1.
SLIDE_MALFORMED = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
       xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:cSld><p:spTree>
    <p:sp>
      <p:nvSpPr><p:nvPr><p:ph type="title"/></p:nvPr></p:nvSpPr>
      <p:txBody><a:p><a:r><a:t>Unclosed title</a:r></a:p></p:txBody>
    </p:sp>
  </p:spTree></p:cSld>
</p:sld>
"""

SHEET_MALFORMED = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>
    <row r="1">
      <c r="A1" t="inlineStr"><is><t>Unclosed</is></c>
    </row>
  </sheetData>
</worksheet>
"""

# ------------------------------------------------------------------ EPUB

EPUB_CONTAINER = """<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles>
</container>
"""

EPUB_OPF = """<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="bookid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Epub Fixture Book</dc:title>
    <dc:identifier id="bookid">urn:uuid:fixture</dc:identifier>
  </metadata>
  <manifest>
    <item id="c1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
    <item id="c2" href="chapter2.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="c1"/>
    <itemref idref="c2"/>
  </spine>
</package>
"""

EPUB_CHAPTER = """<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml">
  <head><title>{title}</title></head>
  <body>
    <h1>{title}</h1>
    <p>{body}</p>
  </body>
</html>
"""

EPUB_CHAPTER_MALFORMED = """<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml">
  <head><title>Broken</title></head>
  <body>
    <h1>Broken chapter</h1>
    <p>An unclosed paragraph.
  </body>
</html>
"""

# -------------------------------------------------------------------- PDF

def pdf_bytes(objects, root_obj):
    """Assemble a minimal PDF from 1-indexed object bodies, with a real xref."""
    out = bytearray(b"%PDF-1.4\n")
    offsets = []
    for index, body in enumerate(objects, start=1):
        offsets.append(len(out))
        out += f"{index} 0 obj\n".encode() + body + b"\nendobj\n"
    xref_at = len(out)
    out += f"xref\n0 {len(objects) + 1}\n".encode()
    out += b"0000000000 65535 f \n"
    for offset in offsets:
        out += f"{offset:010d} 00000 n \n".encode()
    out += (
        f"trailer\n<< /Size {len(objects) + 1} /Root {root_obj} 0 R >>\n"
        f"startxref\n{xref_at}\n%%EOF\n"
    ).encode()
    return bytes(out)


def build_pdf_text(path):
    """A text-bearing PDF: one page, one content stream, real Tj operators."""
    stream = (
        b"BT /F1 24 Tf 72 700 Td (Pdf Fixture Heading) Tj ET\n"
        b"BT /F1 12 Tf 72 660 Td (First pdf paragraph.) Tj ET\n"
        b"BT /F1 12 Tf 72 640 Td (Second pdf paragraph.) Tj ET\n"
        b"BT /F1 12 Tf 72 620 Td (Third pdf paragraph.) Tj ET\n"
    )
    objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] "
        b"/Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>",
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
        b"<< /Length " + str(len(stream)).encode() + b" >>\nstream\n" + stream + b"endstream",
    ]
    with open(path, "wb") as f:
        f.write(pdf_bytes(objects, 1))


def build_pdf_scanned(path):
    """An image-only PDF: one page whose only content is an XObject image and
    NO text operator anywhere. This is the scanned-document shape -- the page
    carries pixels, not characters -- and OCR is the only way to read it.
    OCR is deliberately outside 0.3.0, so this fixture exists to prove the
    named coverage gap, never a silent empty extraction."""
    # 8x8 1-bit-per-pixel image, all zero bits: a blank scan.
    image = bytes(8)
    stream = b"q 612 0 0 792 0 0 cm /Im1 Do Q\n"
    objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] "
        b"/Resources << /XObject << /Im1 4 0 R >> >> /Contents 5 0 R >>",
        b"<< /Type /XObject /Subtype /Image /Width 8 /Height 8 /ColorSpace /DeviceGray "
        b"/BitsPerComponent 1 /Length " + str(len(image)).encode() + b" >>\nstream\n"
        + image + b"\nendstream",
        b"<< /Length " + str(len(stream)).encode() + b" >>\nstream\n" + stream + b"endstream",
    ]
    with open(path, "wb") as f:
        f.write(pdf_bytes(objects, 1))


# ------------------------------------------------------------------ build

def write_zip(path, parts, hostile_entry=None):
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        for name, data in parts:
            z.writestr(name, data)
        if hostile_entry is not None:
            name, size = hostile_entry
            z.writestr(name, b"A" * size)


def pptx_parts(slide1, slide2):
    return [
        ("[Content_Types].xml",
         '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'
         '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
         '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
         '<Default Extension="xml" ContentType="application/xml"/></Types>'),
        ("_rels/.rels", PKG_RELS.format(target="ppt/presentation.xml")),
        ("ppt/presentation.xml", PRESENTATION_XML),
        ("ppt/_rels/presentation.xml.rels", PRESENTATION_RELS),
        ("ppt/slides/slide1.xml", slide1),
        ("ppt/slides/slide2.xml", slide2),
    ]


def xlsx_parts(sheet):
    return [
        ("[Content_Types].xml",
         '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'
         '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
         '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
         '<Default Extension="xml" ContentType="application/xml"/></Types>'),
        ("_rels/.rels", PKG_RELS.format(target="xl/workbook.xml")),
        ("xl/workbook.xml", WORKBOOK_XML),
        ("xl/_rels/workbook.xml.rels", WORKBOOK_RELS),
        ("xl/worksheets/sheet1.xml", sheet),
    ]


def epub_parts(chapter2):
    return [
        ("mimetype", "application/epub+zip"),
        ("META-INF/container.xml", EPUB_CONTAINER),
        ("OEBPS/content.opf", EPUB_OPF),
        ("OEBPS/chapter1.xhtml",
         EPUB_CHAPTER.format(title="Epub Chapter One", body="First epub paragraph.")),
        ("OEBPS/chapter2.xhtml", chapter2),
    ]


# One giant repeated-character entry: DEFLATE compresses it to a few hundred
# KB on disk while claiming 140 MiB uncompressed -- past anydoc's own
# `package::limits::MAX_ENTRY_BYTES` (128 MiB) and comfortably under the
# supervised worker's 512 MiB RLIMIT_AS, exactly as the docx corpus's
# `06-hostile-entry-expansion.docx` is sized and for the same reason.
HOSTILE_ENTRY_BYTES = 140 * 1024 * 1024


def main(out_dir):
    os.makedirs(out_dir, exist_ok=True)
    j = lambda name: os.path.join(out_dir, name)

    with open(j("07-rtf-plain.rtf"), "w", newline="") as f:
        f.write(RTF_PLAIN)
    # Byte-identical to 07, under a `.doc` extension: anydoc's own dispatcher
    # routes `{\rtf`-prefixed `.doc` bytes to the RTF frontend ("RTF files
    # wearing a .doc extension are common in the wild" -- formats/mod.rs).
    with open(j("08-doc-rtf-in-disguise.doc"), "w", newline="") as f:
        f.write(RTF_PLAIN)
    with open(j("09-rtf-deep-nesting.rtf"), "w", newline="") as f:
        f.write(RTF_DEEP_NESTING)

    write_odf(j("10-odt-headings.odt"), "odt", ODT_CONTENT)
    write_odf(j("11-ods-sheet.ods"), "ods", ODS_CONTENT)
    write_odf(j("12-odp-slides.odp"), "odp", ODP_CONTENT)
    write_odf(j("13-odt-malformed-unclosed-element.odt"), "odt", ODT_MALFORMED)
    write_odf(j("14-odt-encrypted.odt"), "odt", ODT_CONTENT,
              manifest=ODF_ENCRYPTED_MANIFEST)

    write_zip(j("15-pptx-slides.pptx"), pptx_parts(
        SLIDE_XML.format(title="Pptx Slide One", body="Pptx first bullet."),
        SLIDE_XML.format(title="Pptx Slide Two", body="Pptx second bullet."),
    ))
    write_zip(j("16-pptx-malformed-unclosed-element.pptx"), pptx_parts(
        SLIDE_MALFORMED,
        SLIDE_XML.format(title="Pptx Slide Two", body="Pptx second bullet."),
    ))
    # The oversized entry has to be a part the frontend actually READS.
    # An unread `ppt/media/*.bin` entry is never decompressed, so a zip bomb
    # parked there proves nothing about the parser's bounds -- verified
    # empirically: the first version of this fixture put it there and the
    # document parsed clean. `slide1.xml` is read.
    write_zip(
        j("17-pptx-hostile-entry-expansion.pptx"),
        [
            part for part in pptx_parts(
                SLIDE_XML.format(title="Pptx Slide One", body="Pptx first bullet."),
                SLIDE_XML.format(title="Pptx Slide Two", body="Pptx second bullet."),
            ) if part[0] != "ppt/slides/slide1.xml"
        ],
        hostile_entry=("ppt/slides/slide1.xml", HOSTILE_ENTRY_BYTES),
    )

    write_zip(j("18-xlsx-sheet.xlsx"), xlsx_parts(SHEET_XML))
    write_zip(j("19-xlsx-malformed-unclosed-element.xlsx"), xlsx_parts(SHEET_MALFORMED))

    write_zip(j("20-epub-chapters.epub"), epub_parts(
        EPUB_CHAPTER.format(title="Epub Chapter Two", body="Second epub paragraph.")))
    write_zip(j("21-epub-malformed-unclosed-element.epub"),
              epub_parts(EPUB_CHAPTER_MALFORMED))

    build_pdf_text(j("22-pdf-text.pdf"))
    build_pdf_scanned(j("23-pdf-scanned-needs-ocr.pdf"))

    for name in sorted(os.listdir(out_dir)):
        print(f"{name}\t{os.path.getsize(os.path.join(out_dir, name))}")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "office_fixtures")
