#!/usr/bin/env python3
"""Builds the Y4/G4 mail_corpus .eml fixtures byte-exactly, and prints the
plaintext values used to construct any encoded content (RFC2047 headers,
quoted-printable, base64) so the manifest's "expected" counts can be written
down BEFORE anything is decoded by a library. This script only ENCODES known
plaintext into the wire format -- it does not parse/decode anything, and it
does not invoke mail-parser. Independent cross-check of the decode direction
happens separately via Python's stdlib `email` package (see
verify_with_stdlib_email.py), which is a different implementation from both
this encoder and from mail-parser.
"""
import base64
import quopri
import os

OUT = "/var/tmp/hats4/y4/tests/fixtures/mail_corpus"
CRLF = "\r\n"


def w(name, parts):
    """parts: list of str (CRLF-joined) or bytes (written raw)."""
    path = os.path.join(OUT, name)
    with open(path, "wb") as f:
        for p in parts:
            if isinstance(p, str):
                f.write(p.encode("utf-8"))
            else:
                f.write(p)
    print(f"wrote {path} ({os.path.getsize(path)} bytes)")


# ---------------------------------------------------------------- fixture 01
f01 = CRLF.join([
    "From: Alice Adams <alice@example.com>",
    "To: Bob Bianchi <bob@example.com>",
    "Subject: Plain text status update",
    "Date: Mon, 12 Jan 2026 09:15:00 +0000",
    "Message-ID: <fixture01-plain@example.com>",
    "Content-Type: text/plain; charset=us-ascii",
    "Content-Transfer-Encoding: 7bit",
    "",
    "Hello Bob,",
    "",
    "This is a plain text message with no MIME structure beyond the single",
    "text/plain body. Three paragraphs total, this being the second.",
    "",
    "Regards,",
    "Alice",
    "",
])
w("01-plain-text.eml", [f01])

# ---------------------------------------------------------------- fixture 02
f02 = CRLF.join([
    "From: Carol Chen <carol@example.com>",
    "To: Dave Diallo <dave@example.com>, Elena Petrova <elena@example.com>",
    "Cc: Frank Osei <frank@example.com>",
    "Subject: Alternative body demo",
    "Date: Tue, 13 Jan 2026 11:00:00 +0000",
    "Message-ID: <fixture02-alt@example.com>",
    "MIME-Version: 1.0",
    'Content-Type: multipart/alternative; boundary="ALT-BOUNDARY-02"',
    "",
    "This is a multi-part message in MIME format.",
    "--ALT-BOUNDARY-02",
    "Content-Type: text/plain; charset=us-ascii",
    "Content-Transfer-Encoding: 7bit",
    "",
    "Plain-text alternative body.",
    "--ALT-BOUNDARY-02",
    "Content-Type: text/html; charset=us-ascii",
    "Content-Transfer-Encoding: 7bit",
    "",
    "<html><body><p>HTML alternative body.</p></body></html>",
    "--ALT-BOUNDARY-02--",
    "",
])
w("02-multipart-alternative.eml", [f02])

# ---------------------------------------------------------------- fixture 03
attach_plain = "Report body line one.\nReport body line two."
attach_b64 = base64.b64encode(attach_plain.encode("ascii")).decode("ascii")
f03 = CRLF.join([
    "From: Grace Grant <grace@example.com>",
    "To: Henry Huang <henry@example.com>",
    "Subject: Report attached",
    "Date: Wed, 14 Jan 2026 08:30:00 +0000",
    "Message-ID: <fixture03-attach@example.com>",
    "MIME-Version: 1.0",
    'Content-Type: multipart/mixed; boundary="MIX-BOUNDARY-03"',
    "",
    "--MIX-BOUNDARY-03",
    "Content-Type: text/plain; charset=us-ascii",
    "Content-Transfer-Encoding: 7bit",
    "",
    "Please find the attached report.",
    "--MIX-BOUNDARY-03",
    'Content-Type: text/plain; name="report.txt"',
    'Content-Disposition: attachment; filename="report.txt"',
    "Content-Transfer-Encoding: base64",
    "",
    attach_b64,
    "--MIX-BOUNDARY-03--",
    "",
])
w("03-with-attachment.eml", [f03])
print(f"[03] attachment plaintext = {attach_plain!r} ({len(attach_plain.encode('ascii'))} bytes)")
print(f"[03] attachment base64    = {attach_b64!r}")

# ---------------------------------------------------------------- fixture 04
inner_body = "This is the original message text, now nested one level deep."
inner = CRLF.join([
    "From: Karen Kowalski <karen@example.com>",
    "To: Irene Ionescu <irene@example.com>",
    "Subject: Original note",
    "Date: Wed, 14 Jan 2026 09:00:00 +0000",
    "Message-ID: <fixture04-inner@example.com>",
    "Content-Type: text/plain; charset=us-ascii",
    "Content-Transfer-Encoding: 7bit",
    "",
    inner_body,
    "",
])
f04 = CRLF.join([
    "From: Irene Ionescu <irene@example.com>",
    "To: Jack Jensen <jack@example.com>",
    "Subject: Forwarded thread",
    "Date: Thu, 15 Jan 2026 10:00:00 +0000",
    "Message-ID: <fixture04-outer@example.com>",
    "MIME-Version: 1.0",
    'Content-Type: multipart/mixed; boundary="NEST-BOUNDARY-04"',
    "",
    "--NEST-BOUNDARY-04",
    "Content-Type: text/plain; charset=us-ascii",
    "Content-Transfer-Encoding: 7bit",
    "",
    "See the forwarded message below.",
    "--NEST-BOUNDARY-04",
    "Content-Type: message/rfc822",
    'Content-Disposition: attachment; filename="original.eml"',
    "",
]) + CRLF + inner + CRLF + "--NEST-BOUNDARY-04--" + CRLF
w("04-nested-rfc822.eml", [f04])

# ---------------------------------------------------------------- fixture 05
# RFC 2047 B-encoded UTF-8 Subject.
subject_plain = "Café update ☕"  # "Café update ☕"
subject_b64 = base64.b64encode(subject_plain.encode("utf-8")).decode("ascii")
subject_encoded_word = f"=?UTF-8?B?{subject_b64}?="

# RFC 2047 Q-encoded ISO-8859-1 display name in From.
from_display_plain = "René Dupont"  # "René Dupont"
from_display_q = quopri.encodestring(
    from_display_plain.encode("iso-8859-1"), header=True
).decode("ascii")
from_encoded_word = f"=?ISO-8859-1?Q?{from_display_q}?="

# Body: text/plain, quoted-printable, non-UTF-8 charset (windows-1252),
# containing a character (e) with an accent outside plain ASCII.
body_plain = "Prix unitaire: 12€ le café.\nTotal: 24€."
# windows-1252 does not encode U+20AC (Euro) the same as ISO-8859-1 -- it's
# byte 0x80 in cp1252, which IS representable, so this also proves a
# non-Latin-1 legacy charset is exercised, not just accented Latin letters.
body_cp1252 = body_plain.encode("cp1252")
body_qp = quopri.encodestring(body_cp1252).decode("ascii")

# Attachment: small binary (non-text) payload, base64, in the same message.
attach_bytes = bytes([0x00, 0x01, 0x02, 0xFF, 0xFE, 0x10, 0x20, 0x30])
attach_b64_05 = base64.b64encode(attach_bytes).decode("ascii")

f05 = CRLF.join([
    f"From: {from_encoded_word} <rene@example.com>",
    "To: Alice Adams <alice@example.com>",
    f"Subject: {subject_encoded_word}",
    "Date: Fri, 16 Jan 2026 12:00:00 +0000",
    "Message-ID: <fixture05-zoo@example.com>",
    "MIME-Version: 1.0",
    'Content-Type: multipart/mixed; boundary="ZOO-BOUNDARY-05"',
    "",
    "--ZOO-BOUNDARY-05",
    "Content-Type: text/plain; charset=windows-1252",
    "Content-Transfer-Encoding: quoted-printable",
    "",
]) + CRLF + body_qp + CRLF + CRLF.join([
    "--ZOO-BOUNDARY-05",
    'Content-Type: application/octet-stream; name="blob.bin"',
    'Content-Disposition: attachment; filename="blob.bin"',
    "Content-Transfer-Encoding: base64",
    "",
    attach_b64_05,
    "--ZOO-BOUNDARY-05--",
    "",
])
w("05-encoding-zoo.eml", [f05])
print(f"[05] subject plaintext = {subject_plain!r}")
print(f"[05] subject encoded-word = {subject_encoded_word!r}")
print(f"[05] from-display plaintext = {from_display_plain!r}")
print(f"[05] from-display encoded-word = {from_encoded_word!r}")
print(f"[05] body plaintext = {body_plain!r}")
print(f"[05] body cp1252 bytes = {body_cp1252!r}")
print(f"[05] attachment bytes = {attach_bytes!r} ({len(attach_bytes)} bytes) sha256-not-needed, len only")

# ---------------------------------------------------------------- fixture 06
# Deliberately malformed: NOT email-shaped at all -- no "Name: value" header
# line anywhere before end of content, so mail-parser's documented contract
# ("returns None if no headers are found", verified via ctx7 against
# docs.rs/mail-parser 0.11.8) has zero headers to recover, and the only
# honest outcome is a hard parse failure, never a partial Message.
f06 = "\n".join([
    "This file is not shaped like an RFC 5322 message at all.",
    "There is no header block above a blank-line body separator --",
    "every line here is prose, none of them is a Name-colon-value pair,",
    "so a conformant parser has no header field to recover and must",
    "refuse to produce a Message rather than guess one into existence.",
    "",
])
w("06-malformed-no-headers.eml", [f06])
