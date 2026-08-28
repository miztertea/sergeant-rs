#!/usr/bin/env python3
"""Independent cross-check of the mail_corpus fixtures using Python's
stdlib `email` package -- a different implementation from both the
byte-construction script (build_mail_fixtures.py) and from mail-parser
(the crate under evaluation). Confirms decode direction only; does not
touch mail-parser or Cargo at all.
"""
import email
import email.policy
import os

DIR = "/var/tmp/hats4/y4/tests/fixtures/mail_corpus"


def load(name):
    with open(os.path.join(DIR, name), "rb") as f:
        return email.message_from_binary_file(f, policy=email.policy.default)


def leaf_parts(msg):
    if msg.is_multipart():
        out = []
        for p in msg.iter_parts():
            out.extend(leaf_parts(p))
        return out
    return [msg]


print("=== 01-plain-text.eml ===")
m = load("01-plain-text.eml")
print("headers:", list(m.keys()), "count=", len(m.keys()))
print("is_multipart:", m.is_multipart())
print("content-type:", m.get_content_type())
print("body:", repr(m.get_content()))

print()
print("=== 02-multipart-alternative.eml ===")
m = load("02-multipart-alternative.eml")
print("headers:", list(m.keys()), "count=", len(m.keys()))
print("to-addr-count:", len(m["To"].addresses) if m["To"] else 0)
print("cc-addr-count:", len(m["Cc"].addresses) if m["Cc"] else 0)
leaves = leaf_parts(m)
print("leaf part count:", len(leaves))
for p in leaves:
    print(" -", p.get_content_type(), repr(p.get_content())[:60])

print()
print("=== 03-with-attachment.eml ===")
m = load("03-with-attachment.eml")
print("headers count=", len(m.keys()))
leaves = leaf_parts(m)
print("leaf part count:", len(leaves))
atts = [p for p in leaves if p.get_content_disposition() == "attachment"]
print("attachment count:", len(atts))
for p in atts:
    payload = p.get_content()
    print(" attachment filename:", p.get_filename(), "decoded bytes:", repr(payload), "len=", len(payload) if isinstance(payload, (bytes, str)) else "?")

print()
print("=== 04-nested-rfc822.eml ===")
m = load("04-nested-rfc822.eml")
print("outer headers count=", len(m.keys()))
leaves_top = list(m.iter_parts()) if m.is_multipart() else [m]
print("outer top-level part count:", len(leaves_top))
nested = [p for p in leaves_top if p.get_content_type() == "message/rfc822"]
print("nested message/rfc822 count:", len(nested))
for p in nested:
    inner = p.get_content()  # returns EmailMessage for message/rfc822
    print("  inner headers count=", len(inner.keys()))
    print("  inner subject:", inner["Subject"])
    print("  inner body:", repr(inner.get_content()))

print()
print("=== 05-encoding-zoo.eml ===")
m = load("05-encoding-zoo.eml")
print("headers count=", len(m.keys()))
print("decoded Subject:", m["Subject"].get_content() if hasattr(m["Subject"], "get_content") else str(m["Subject"]))
print("raw Subject header object:", repr(m["Subject"]))
from_header = m["From"]
print("decoded From:", str(from_header))
print("From addresses:", [(a.display_name, a.addr_spec) for a in from_header.addresses])
leaves = leaf_parts(m)
print("leaf part count:", len(leaves))
for p in leaves:
    ct = p.get_content_type()
    if ct == "text/plain":
        print(" text/plain charset param:", p.get_param("charset"))
        print(" text/plain decoded content:", repr(p.get_content()))
    else:
        payload = p.get_payload(decode=True)
        print(" attachment", p.get_filename(), "decoded len=", len(payload), repr(payload))

print()
print("=== 06-malformed-no-headers.eml ===")
m = load("06-malformed-no-headers.eml")
print("headers found:", list(m.keys()), "count=", len(m.keys()))
print("defects:", m.defects)
print("is_multipart:", m.is_multipart())
print("content as body (if any headers were found, first bytes fell into body):")
try:
    print(repr(m.get_content())[:200])
except Exception as e:
    print("  get_content() raised:", e)
