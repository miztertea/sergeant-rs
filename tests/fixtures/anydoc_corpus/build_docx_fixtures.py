#!/usr/bin/env python3
"""Hand-build minimal, hand-verifiable .docx fixtures for S4 Y2 gate (b).

No python-docx, no external dependency: a .docx is just a zip of OOXML XML
parts, written here as literal strings so every element in every fixture is
something a human can read directly (unzip the file, open word/document.xml
in a text editor) and count by hand -- which is the point: the fixture
corpus's counts are hand-verified BEFORE any extractor exists, and this
script is how they were produced, not a black box.

Run: python3 build_docx_fixtures.py <out_dir>
"""
import sys
import zipfile
import os

CONTENT_TYPES = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
{extra_overrides}</Types>
"""

RELS = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>
"""

STYLES = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:style w:type="paragraph" w:styleId="Normal"><w:name w:val="Normal"/></w:style>
  <w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/></w:style>
  <w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:basedOn w:val="Normal"/></w:style>
  <w:style w:type="paragraph" w:styleId="ListParagraph"><w:name w:val="List Paragraph"/><w:basedOn w:val="Normal"/></w:style>
</w:styles>
"""

DOC_RELS_TEMPLATE = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
{extra}</Relationships>
"""

DOCUMENT_TEMPLATE = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
{body}
    <w:sectPr>
      <w:pgSz w:w="12240" w:h="15840"/>
    </w:sectPr>
  </w:body>
</w:document>
"""


def para(text, style=None, numid=None, ilvl=None, footnote_ref=None):
    ppr_parts = []
    if style:
        ppr_parts.append(f'<w:pStyle w:val="{style}"/>')
    if numid is not None:
        ppr_parts.append(f'<w:numPr><w:ilvl w:val="{ilvl}"/><w:numId w:val="{numid}"/></w:numPr>')
    ppr = f"<w:pPr>{''.join(ppr_parts)}</w:pPr>" if ppr_parts else ""
    run = f'<w:r><w:t xml:space="preserve">{text}</w:t></w:r>' if text else ""
    fn = f'<w:r><w:footnoteReference w:id="{footnote_ref}"/></w:r>' if footnote_ref is not None else ""
    return f"    <w:p>{ppr}{run}{fn}</w:p>"


def write_docx(path, body_xml, extra_content_type_overrides="", extra_doc_rels="",
                extra_parts=None):
    extra_parts = extra_parts or {}
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr(
            "[Content_Types].xml",
            CONTENT_TYPES.format(extra_overrides=extra_content_type_overrides),
        )
        z.writestr("_rels/.rels", RELS)
        z.writestr("word/styles.xml", STYLES)
        z.writestr(
            "word/document.xml", DOCUMENT_TEMPLATE.format(body=body_xml)
        )
        z.writestr(
            "word/_rels/document.xml.rels",
            DOC_RELS_TEMPLATE.format(extra=extra_doc_rels),
        )
        for name, content in extra_parts.items():
            z.writestr(name, content)


def build_01_plain(out_dir):
    body = "\n".join([
        para("Introduction", style="Heading1"),
        para("This is the first body paragraph under the introduction heading."),
        para("This is the second body paragraph under the introduction heading."),
        para("Background", style="Heading2"),
        para("A single body paragraph under the background heading."),
    ])
    write_docx(os.path.join(out_dir, "01-plain-headings-paragraphs.docx"), body)


def build_02_nested_list(out_dir):
    numbering = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:abstractNum w:abstractNumId="0">
    <w:lvl w:ilvl="0"><w:numFmt w:val="decimal"/></w:lvl>
    <w:lvl w:ilvl="1"><w:numFmt w:val="lowerLetter"/></w:lvl>
  </w:abstractNum>
  <w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num>
</w:numbering>
"""
    body = "\n".join([
        para("Shopping list", style="Heading1"),
        para("Produce", numid=1, ilvl=0, style="ListParagraph"),
        para("Apples", numid=1, ilvl=1, style="ListParagraph"),
        para("Bananas", numid=1, ilvl=1, style="ListParagraph"),
        para("Dairy", numid=1, ilvl=0, style="ListParagraph"),
        para("Milk", numid=1, ilvl=1, style="ListParagraph"),
    ])
    write_docx(
        os.path.join(out_dir, "02-nested-list-numbering.docx"),
        body,
        extra_content_type_overrides='  <Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/>\n',
        extra_doc_rels='  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/>\n',
        extra_parts={"word/numbering.xml": numbering},
    )


def build_03_table(out_dir):
    def cell(text):
        return f'<w:tc><w:p><w:r><w:t xml:space="preserve">{text}</w:t></w:r></w:p></w:tc>'

    def row(cells):
        return f"<w:tr>{''.join(cell(c) for c in cells)}</w:tr>"

    table = (
        "<w:tbl>"
        "<w:tblPr/>"
        "<w:tblGrid><w:gridCol/><w:gridCol/></w:tblGrid>"
        + row(["Name", "Quantity"])
        + row(["Widget", "12"])
        + row(["Gadget", "7"])
        + "</w:tbl>"
    )
    body = "\n".join([
        para("Inventory", style="Heading1"),
        para("The table below lists current stock."),
        table,
        para("End of inventory report."),
    ])
    write_docx(os.path.join(out_dir, "03-table.docx"), body)


def build_04_footnotes_headers_footers(out_dir):
    footnotes = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote>
  <w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote>
  <w:footnote w:id="1"><w:p><w:r><w:t xml:space="preserve">First footnote text.</w:t></w:r></w:p></w:footnote>
  <w:footnote w:id="2"><w:p><w:r><w:t xml:space="preserve">Second footnote text.</w:t></w:r></w:p></w:footnote>
</w:footnotes>
"""
    header = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:p><w:r><w:t xml:space="preserve">Quarterly Report -- Header</w:t></w:r></w:p>
</w:hdr>
"""
    footer = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:p><w:r><w:t xml:space="preserve">Page footer -- confidential</w:t></w:r></w:p>
</w:ftr>
"""
    body = "\n".join([
        para("Findings", style="Heading1"),
        para("The first claim needing a citation.", footnote_ref=1),
        para("The second claim needing a citation.", footnote_ref=2),
    ])
    write_docx(
        os.path.join(out_dir, "04-footnotes-headers-footers.docx"),
        body,
        extra_content_type_overrides=(
            '  <Override PartName="/word/footnotes.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml"/>\n'
            '  <Override PartName="/word/header1.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml"/>\n'
            '  <Override PartName="/word/footer1.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml"/>\n'
        ),
        extra_doc_rels=(
            '  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes" Target="footnotes.xml"/>\n'
            '  <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="header1.xml"/>\n'
            '  <Relationship Id="rId4" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" Target="footer1.xml"/>\n'
        ),
        extra_parts={
            "word/footnotes.xml": footnotes,
            "word/header1.xml": header,
            "word/footer1.xml": footer,
        },
    )


def build_05_malformed(out_dir):
    # Deliberately malformed: a syntactically valid zip/OOXML package whose
    # word/document.xml has an unclosed element -- not truncated bytes (which
    # would just be a corrupt-zip test), a structurally invalid *document*,
    # which is the failure mode a real extractor actually has to detect and
    # refuse rather than partially walk.
    bad_document = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r><w:t>This paragraph's element is never closed
    <w:p><w:r><w:t xml:space="preserve">A second paragraph that a partial/tolerant parser might wrongly still yield.</w:t></w:r></w:p>
  </w:body>
</w:document>
"""
    with zipfile.ZipFile(
        os.path.join(out_dir, "05-malformed-unclosed-element.docx"), "w", zipfile.ZIP_DEFLATED
    ) as z:
        z.writestr("[Content_Types].xml", CONTENT_TYPES.format(extra_overrides=""))
        z.writestr("_rels/.rels", RELS)
        z.writestr("word/styles.xml", STYLES)
        z.writestr("word/document.xml", bad_document)
        z.writestr("word/_rels/document.xml.rels", DOC_RELS_TEMPLATE.format(extra=""))


if __name__ == "__main__":
    out_dir = sys.argv[1]
    os.makedirs(out_dir, exist_ok=True)
    build_01_plain(out_dir)
    build_02_nested_list(out_dir)
    build_03_table(out_dir)
    build_04_footnotes_headers_footers(out_dir)
    build_05_malformed(out_dir)
    print("built fixtures in", out_dir)
