//! Mail (`.eml`) adapter (S4 Y4, G4 — ADOPT, `tests/fixtures/mail_corpus/SPIKE-G4.md`).
//!
//! ```text
//! bytes  ──►  mail_parser::MessageParser  ──►  MailMessage (our own vocabulary)
//! ```
//!
//! A pure function over bytes (F6), meant to run inside Y1's supervised
//! worker exactly the way `office::docx_units`/`archive::expand` already do.
//! Message shape follows A1 §6.5 verbatim: from/to/cc, sent timestamp,
//! subject, text AND html bodies, message id and thread/references ids,
//! attachments. Provenance carries parser identity + version
//! ([`MAIL_EXTRACTOR`]).
//!
//! # Two caveats the G4 spike found empirically, CLOSED here — not merely
//! named as downstream work
//!
//! `SPIKE-G4.md` gate (b) found two gaps in `mail-parser` 0.11.8's own
//! behavior and named them as "downstream wave work, not built in this
//! spike." This module is that downstream wave. Both are closed by reading
//! `mail-parser` 0.11.8's own source directly
//! (`~/.cargo/registry/.../mail-parser-0.11.8/src/`, not merely its docs or
//! the spike's empirical transcript), because the spike's own caveat is that
//! this crate's *documented* Robustness Principle already told us silent
//! degradation was possible — closing it honestly needs the actual
//! mechanism, not another empirical probe.
//!
//! ## 1. Synthesized HTML/text bodies (caveat 2) — VERIFIED against
//! `parsers/message.rs`'s own synthesis mechanism, not merely observed
//!
//! **Finding**: `mail-parser` never allocates a second, synthetic
//! [`mail_parser::MessagePart`] for a synthesized body. It pushes the
//! **identical part index** onto both `Message::html_body` and
//! `Message::text_body`. Two code paths do this, both verified directly:
//! (a) a single-part message with no `multipart/alternative` at all
//! (`state.parts == 1`) sets `(add_to_html, add_to_text)` from
//! `(state.need_html_body, state.need_text_body)`, both of which default
//! `true` and are never narrowed absent an alternative — so a bare
//! `text/plain` message's one part is pushed onto *both* vectors; (b) a
//! `multipart/alternative` that supplies only one of the two — after the
//! part loop, `parsers/message.rs`'s own "Found HTML part only"/"Found text
//! part only" blocks copy the found vector's part ids into the *missing*
//! vector verbatim. Either way, the tell is identical and checkable without
//! inspecting content at all: **the same [`mail_parser::MessagePartId`]
//! present at the front of both `text_body` and `html_body` means the "HTML"
//! is an alias of the plain-text part, not a part the wire bytes actually
//! declared `Content-Type: text/html`.** [`genuine_html`] is exactly this
//! check. `manifest.json`'s own `body_html_present` field is defined against
//! the wire bytes for the identical reason (`MANIFEST.md`'s own
//! `counting_rules`), so this module's `html_body: None` on a synthesized
//! case is what keeps [`MailMessage`] answering the *same* question the
//! fixture corpus's own ground truth answers.
//!
//! ## 2. Silent degraded-parse recovery (caveat 1) — a narrow, stated
//! heuristic, not a general strict mode
//!
//! `mail-parser` has no strict mode (confirmed against its own source: no
//! `pkcs7`/`smime`/`encrypted` awareness either, see "Sealed detection"
//! below, and no boundary-well-formedness signal of any kind is surfaced —
//! unlike the third-party document-conversion crate whose own recovery
//! event `office.rs`'s own `RecoveryWatch` intercepts (that crate is named
//! nowhere outside `office.rs` itself, `tests/y2_office_boundary.rs`'s own
//! structural pin — this module does not name it either), `mail-parser`
//! pulls in no `log` dependency at all: its `Cargo.toml`'s own feature
//! table names none). The spike's own
//! diagnostic transcript (`diagnostic-not-manifest-broken-mime.eml`,
//! `SPIKE-G4.md` gate (b)) is this module's only evidence for the failure
//! *shape*, and the shape is narrow and specific: the one body part that
//! should have been `text/plain` came back as a leaf in
//! `Message::attachments` whose `attachment_name()` is `None` —
//! `attachment[0] name=None len=48`, verbatim. A genuinely-authored
//! attachment overwhelmingly carries a `Content-Disposition: filename=` or a
//! `Content-Type: name=` (`attachment_name`'s own two-step lookup,
//! `mail-parser`'s `lib.rs`), so [`degraded_body_part`] treats **any leaf,
//! non-`message/rfc822` attachment with no recoverable name** as evidence
//! the parse degraded rather than genuinely producing an unnamed
//! attachment, and refuses the WHOLE message as [`MailError::Degraded`] —
//! no partial units (F8; brief item 5). **Named, accepted limitation, not
//! glossed over**: a legitimately-authored message really can omit a
//! filename on both headers (rare, but RFC-legal); this check
//! misclassifies that case as degraded. Kept rather than closed for the
//! same reason `archive.rs`'s own case-folding gap is kept: the
//! authoritative defense this build has for "coverage-honest degraded
//! parse" is exactly this signal, and refusing a genuinely rare, genuinely
//! ambiguous case costs less than silently trusting a parse that already
//! demonstrated it can drop structure without any error at all. A
//! `message/rfc822` attachment is EXCLUDED from this check (a nested
//! message legitimately carries no filename far more often — see "Container
//! recursion" below), and is instead refused per-item (empty name) exactly
//! as an ordinary leaf attachment with no name would be under Y3's own
//! admission discipline, never poisoning the outer message.
//!
//! # Sealed (encrypted/S-MIME) messages get their own honest status
//! (brief item 5)
//!
//! `mail-parser` 0.11.8 has **zero** PKCS#7/S-MIME awareness of any kind
//! (VERIFIED: `grep -rin 'pkcs7\|smime\|encrypted' src/` against the crate's
//! own source returns nothing) — an `application/pkcs7-mime` or
//! `multipart/encrypted` message parses as ordinary opaque binary/multipart
//! content, which would otherwise read as a message with a garbage body
//! rather than the honest "cannot read this without decrypting/verifying it,
//! which this build does not do" the brief requires. [`sealed_kind`] detects
//! it structurally, from the message's OWN top-level declared Content-Type
//! (`multipart/encrypted`, RFC 1847; `application/pkcs7-mime`/
//! `application/x-pkcs7-mime`, RFC 8551) — never by attempting to decrypt or
//! verify anything, which Sergeant has no key material to do. Deliberately
//! **not** applied to `multipart/signed`/`application/pkcs7-signature`: a
//! signed-but-unencrypted message's content is genuinely readable (the
//! signature rides alongside, not instead of, the plaintext), so treating it
//! as sealed would be dishonest in the other direction.
//!
//! # Container recursion is one shared budget with `archive.rs` (R2)
//!
//! A `message/rfc822` attachment recurses via this module's own
//! [`parse_at_depth`]; an attachment that is itself a ZIP recurses via
//! [`super::archive::expand_at_depth`] — the SAME function `archive.rs`
//! calls on itself for an archive nested inside an archive, called here with
//! the SAME shared `depth`/`cumulative_expanded_bytes` this module's own
//! recursion already threads. This is a deliberate design choice, not an
//! oversight: the brief's "the nesting-depth bound applies to it as it does
//! to nested archives" is read as *the same bound*, not a second
//! independently-sized one, so a mail-inside-a-zip-inside-a-mail chain is
//! bounded by one [`super::archive::MAX_NESTING_DEPTH`] and one
//! [`super::archive::MAX_TOTAL_EXPANDED_BYTES`] across the WHOLE tree,
//! whichever container kind each level happens to be. `super::archive`'s own
//! module doc records the matching half of this reuse.
//!
//! # An honest gap this module does NOT claim to close: post-decode, not
//! pre-allocation, bounds
//!
//! `archive.rs`'s [`super::archive::MAX_ENTRY_UNCOMPRESSED_BYTES`] bound is
//! enforced by `Read::take` — the entry is never allocated past the cap in
//! the first place. `mail-parser` has no equivalent streaming decode API: it
//! parses and decodes every part into an owned `Cow` eagerly, before this
//! adapter's own code ever runs (VERIFIED against its own source: `Message`/
//! `MessagePart` are built in one pass with `PartType::{Text,Html,Binary}`
//! already holding decoded bytes). So every size bound in this module —
//! reused outright from `archive.rs` (R2): [`super::archive::MAX_ENTRY_UNCOMPRESSED_BYTES`]
//! per attachment, [`super::archive::MAX_TOTAL_EXPANDED_BYTES`] cumulative,
//! [`super::archive::MAX_ZIP_ENTRIES`] reused as an attachment-count ceiling
//! — is a POST-decode admission check: it refuses to ADMIT what `mail-parser`
//! already decoded, it cannot prevent the decode itself from allocating. The
//! real backstop against a single oversized MIME part is the SAME one Y1
//! already built for exactly this class of gap: the worker's own
//! [`super::worker::WORKER_ADDRESS_SPACE_LIMIT_BYTES`] `RLIMIT_AS`, armed on
//! every parse-worker child regardless of which adapter runs inside it. This
//! is stated plainly rather than implied away, matching this sprint's own
//! "prose must match code exactly" rule.
//!
//! Named separately, and considered, not merely inherited: MIME transfer
//! encodings (base64, quoted-printable) do not have the ZIP-class
//! decompression-bomb shape `archive.rs`'s own streaming bound defends
//! against — base64 expands input by a fixed ~4:3, quoted-printable by at
//! most ~3:1 in the worst case, neither remotely near `deflate`'s achievable
//! ratios — so the ABSENCE of a pre-allocation stream bound here is a
//! materially smaller gap than it would be for a compressed container, not
//! an equivalent one merely left unfixed.
//!
//! # The same admission discipline as Y3 (brief item 3) — reused, not
//! reinvented (R2)
//!
//! Attachment filenames are attacker-controlled exactly as ZIP entry names
//! are (a `Content-Disposition: filename="../../etc/passwd"` is legal MIME
//! syntax). [`parse_at_depth`]'s attachment loop applies the identical rule
//! set `archive.rs`'s own module doc states for entries, reusing its actual
//! functions rather than a second copy: empty-name refusal; path safety per
//! `/`-separated component via [`crate::domain::is_plain_name`] (the same
//! guard [`super::worker::validate_batch`]'s own `enclosed_relative_path`
//! composes for a worker-declared child path — R2, one guard, not a second
//! one written for mail); name uniqueness at this message's own level; the
//! Unicode NFC-then-case-fold collision rule via
//! [`super::archive::collision_key`] (reused outright, R2 — one
//! normalisation rule for every container this build has, not two that
//! could disagree). And, like `archive.rs`, this module never writes an
//! attachment to a real path at all — everything here is bytes in, structs
//! out — so "never write an attachment anywhere something could execute it"
//! holds structurally, the same way `archive.rs`'s own module doc states it
//! for ZIP entries.
//!
//! # Coverage honesty (brief item 5)
//!
//! [`parse_message`] returns `Err` for a message this build cannot honestly
//! read — [`MailError::Unparseable`] (`mail-parser`'s own documented
//! contract: zero headers found), [`MailError::Sealed`] (structurally
//! encrypted/S-MIME), [`MailError::Degraded`] (the caveat-1 signal) — never
//! a `MailMessage` with partial fields standing in for a refusal. A caller
//! (the worker binary) turns that `Err` into a non-zero exit exactly as
//! `office::docx_units`'s own failure does, which the daemon-side transport
//! already turns into a named [`crate::domain::source::Coverage::Error`]
//! row (R2 — the SAME "a failed extraction exits non-zero, never an empty
//! batch" rule `atlas_worker.rs`'s own module doc states for Office).
//!
//! # No replaceability boundary for `mail_parser` (J1, stated per the
//! brief's own instruction to say why or why not)
//!
//! Office's boundary (`office.rs`'s own module doc, `tests/y2_office_boundary.rs`)
//! exists because the owner's adoption of its own third-party
//! document-conversion crate was conditioned on one, specifically because
//! it crossed a RUSTSEC advisory — the boundary is what
//! makes that acceptance *reversible*. `mail-parser`'s own G4 deny gate
//! found no such advisory (`SPIKE-G4.md` gate (a): zero new
//! advisory/license/ban/source failure), so there is no owner ruling this
//! module's own boundary would be discharging. `archive.rs` (Y3) set the
//! precedent this module follows: a second real container adapter,
//! `zip`, also carries no dedicated one-owner structural test
//! (`tests/y3_zip_adapter.rs` has no `*_boundary.rs` sibling). G9's "one-owner
//! discipline unchanged" is read, consistently with that precedent, as
//! "the two EXISTING structural tests (the database driver's, the Office
//! adapter's own third-party-crate boundary) stay green" — not "every
//! adapter this sprint adds earns a new one." This
//! is a local, reversible choice (J1): nothing downstream depends on
//! `mail_parser` being named nowhere else, and adding a boundary test later
//! costs nothing this decision forecloses.

use std::collections::{BTreeMap, BTreeSet};

use mail_parser::{Address, HeaderValue, MessageParser, MessagePart, MimeHeaders};

use crate::domain::is_plain_name;
use crate::domain::source::{Coverage, CoverageRow, child_key, content_hash};
use crate::runtime::atlas::archive::{
    self, MAX_ENTRY_UNCOMPRESSED_BYTES, MAX_NESTING_DEPTH, MAX_TOTAL_EXPANDED_BYTES,
    MAX_ZIP_ENTRIES, UNSUPPORTED_CHILD_EXTRACTOR,
};

// ------------------------------------------------------------- routing

/// Extractor identity + version (F7's second cache-key input, and A1 §6.3's
/// "normalizer identity" provenance requirement) — versioned exactly as
/// [`super::office::DOCX_EXTRACTOR`]'s own doc explains for its own tag.
pub const MAIL_EXTRACTOR: &str = "mail-parser/0.11.8+eml/v1";

/// Extensions routed to [`parse_message`]. `.eml` only — the contract names
/// it as the first format and G10 defers `.msg`/other mail formats
/// explicitly (brief, "NOT in scope").
pub const MAIL_EXTENSIONS: &[&str] = &["eml"];

/// The extractor for a path, by extension — mirrors
/// [`super::office::extractor_for`]/[`super::archive::extractor_for`]'s own
/// shape exactly.
pub fn extractor_for(relative: &str) -> Option<&'static str> {
    let extension = std::path::Path::new(relative)
        .extension()
        .and_then(|e| e.to_str())?
        .to_ascii_lowercase();
    MAIL_EXTENSIONS
        .contains(&extension.as_str())
        .then_some(MAIL_EXTRACTOR)
}

// ------------------------------------------------------------ output shape

/// One address, in our own vocabulary — no `mail_parser::Addr` crosses this
/// module's boundary into a caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailAddress {
    /// The display name, when present (RFC 2047-decoded already by
    /// `mail-parser`).
    pub name: Option<String>,
    /// The email address, when present.
    pub address: Option<String>,
}

/// One admitted attachment — a child resource exactly as an archive entry
/// is (module doc, "The same admission discipline as Y3").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailAttachment {
    /// The attachment's own name, exactly as `attachment_name()` produced
    /// it — validated (module doc's admission discipline) before this
    /// struct ever exists; a refused candidate never reaches this type,
    /// only its own named [`CoverageRow`] does.
    pub filename: String,
    /// `type/subtype`, when the part declares a Content-Type.
    pub content_type: Option<String>,
    /// The attachment's own decoded bytes — for a `message/rfc822`
    /// attachment, the nested message's own raw bytes
    /// (`mail_parser::Message::raw_message`), bounded by
    /// [`MAX_ENTRY_UNCOMPRESSED_BYTES`] (module doc's own "honest gap":
    /// this is a post-decode admission check, not a pre-allocation stream
    /// bound).
    pub content: Vec<u8>,
    /// BLAKE3 hex of `content` — F7's content half.
    pub content_hash: String,
    /// F7's composed child key (G9) — [`child_key`] applied to
    /// `(parent_key, filename, content_hash, extractor)`, chained from the
    /// immediate parent's own key exactly as [`super::archive::ZipChild::key`]'s
    /// own doc explains.
    pub key: String,
    /// Whether this attachment is itself a nested `message/rfc822`.
    pub is_message: bool,
    /// `Some` when `is_message` and [`MAX_NESTING_DEPTH`] allowed recursing
    /// into it AND the nested message itself parsed cleanly — an inner
    /// message that fails its OWN parse (sealed, degraded, unparseable)
    /// does not poison this outer message; its own failure is named in
    /// `nested_message_error` instead.
    pub nested_message: Option<Box<MailMessage>>,
    /// `Some` naming why recursion into a `message/rfc822` attachment did
    /// not produce `nested_message` — the depth ceiling, or the inner
    /// message's own [`MailError`], stringified. The attachment is still
    /// admitted (hash, key) either way; only its own recursive expansion is
    /// affected, the same "still a child, just not opened further" shape
    /// [`super::archive`]'s own `MAX_NESTING_DEPTH` bullet documents.
    pub nested_message_error: Option<String>,
    /// Whether this attachment is itself a ZIP archive, by extension
    /// ([`super::archive::extractor_for`], via [`super::archive::classify`]).
    pub is_archive: bool,
    /// `Some` when `is_archive` and [`MAX_NESTING_DEPTH`] allowed opening it
    /// — the recursive expansion of this attachment's own bytes through
    /// [`super::archive::expand_at_depth`] (module doc, "Container
    /// recursion is one shared budget").
    pub nested_archive: Option<Box<archive::ZipExpansion>>,
    /// The attachment's own downstream extractor identity when it is
    /// neither a nested message nor an archive — `None` when nothing in
    /// this build claims its extension yet.
    pub entry_adapter: Option<&'static str>,
}

/// One parsed message, in our own vocabulary — A1 §6.5's shape exactly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailMessage {
    /// `From:` addresses (almost always one; RFC 5322 permits more).
    pub from: Vec<MailAddress>,
    /// `To:` addresses, flattened across any RFC 5322 group syntax
    /// (`Address::iter`'s own behavior).
    pub to: Vec<MailAddress>,
    /// `Cc:` addresses, same flattening.
    pub cc: Vec<MailAddress>,
    /// The `Date:` header, RFC3339, when present and parseable.
    pub sent: Option<String>,
    /// The `Subject:` header, RFC 2047-decoded, when present.
    pub subject: Option<String>,
    /// The genuine `text/plain` body, when this message has one.
    pub text_body: Option<String>,
    /// The genuine `text/html` body — **never** a body `mail-parser`
    /// synthesized from a plain-text-only source (module doc, caveat 1;
    /// [`genuine_html`]).
    pub html_body: Option<String>,
    /// The `Message-ID:` header, when present.
    pub message_id: Option<String>,
    /// Thread identifiers: every id in `References:`, then any id in
    /// `In-Reply-To:` not already present — both are "the thread" for A1
    /// §6.5's purposes, and folding them into one ordered, deduplicated
    /// list is simpler than exposing two vectors a caller would just
    /// concatenate itself.
    pub references: Vec<String>,
    /// Every admitted attachment at this message's own level (not counting
    /// grandchildren nested inside an attachment's own `nested_message`/
    /// `nested_archive`).
    pub attachments: Vec<MailAttachment>,
    /// Every attachment-admission refusal this level's own walk produced —
    /// never a silent skip, the same discipline
    /// [`super::archive::ZipExpansion::coverage`] keeps for archive
    /// entries.
    pub coverage: Vec<CoverageRow>,
}

/// Why [`parse_message`] could not produce a [`MailMessage`] — in our own
/// vocabulary, never a `mail_parser` type crossing this boundary.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MailError {
    /// `mail-parser`'s own documented contract: zero RFC 5322 header field
    /// lines found, so `MessageParser::parse` returned `None`.
    #[error(
        "message could not be parsed: no RFC 5322 header field found (mail-parser's own \
         documented contract: MessageParser::parse returns None when no headers are found)"
    )]
    Unparseable,
    /// The message's own top-level Content-Type structurally names it
    /// encrypted or S/MIME-sealed (module doc, "Sealed detection") — never
    /// decrypted or verified.
    #[error(
        "message is encrypted/S-MIME-sealed ({0}); mail-parser has no PKCS#7/S-MIME decoding \
         of its own and this build performs no decryption or signature verification, so this \
         content is not text-readable"
    )]
    Sealed(&'static str),
    /// The caveat-1 signal: a leaf, non-message attachment with no
    /// recoverable name — evidence `mail-parser`'s lenient parse silently
    /// downgraded structure it declared (module doc, caveat 1).
    #[error(
        "message parse degraded silently rather than failing (mail-parser has no strict mode): \
         {0}"
    )]
    Degraded(String),
}

// ------------------------------------------------------------- the adapter

/// Parse one `.eml` resource's bytes into a [`MailMessage`], against
/// `parent_key` — the caller's own [`crate::domain::source::local_key`]/
/// [`crate::domain::source::estate_git_key`] for this resource, which every
/// attachment key composes on top of (F7/G9), exactly as
/// [`super::archive::expand`]'s own doc explains for `parent_key`.
///
/// Pure (F6): no file is opened beyond `bytes`, no clock is read, no store
/// is touched. Two calls on equal bytes and an equal `parent_key` are equal
/// — proven directly below, the same way `office.rs`/`archive.rs`'s own
/// purity tests do.
pub fn parse_message(bytes: &[u8], parent_key: &str) -> Result<MailMessage, MailError> {
    let mut cumulative_expanded_bytes: u64 = 0;
    parse_at_depth(bytes, parent_key, 0, &mut cumulative_expanded_bytes)
}

/// Parses `bytes` at the top of one recursion level, then delegates to
/// [`build_mail_message`]. **Only ever called for the top-level resource and
/// for a nested archive's own re-entry into mail** — a `message/rfc822`
/// attachment does NOT come back through here: `mail-parser` already parsed
/// it fully, correctly, as part of parsing its OUTER message (it hands that
/// parse back as `PartType::Message`, `MessagePart::message()`), and
/// re-serializing it to raw bytes via `contents()`/`raw_message()` only to
/// feed it back through a second `MessageParser::parse` pass is a lossy
/// round-trip: proven wrong empirically while building this module's own
/// tests — `raw_message()`'s reconstructed span picked up one extra
/// boundary-adjacent CRLF a second parse then read as part of the body text,
/// disagreeing with `manifest.json`'s own hand-verified, stdlib-`email`-
/// cross-checked answer. [`parse_at_depth`] is for a byte-addressed
/// container boundary (the resource itself, or a ZIP entry recursing back
/// into mail); [`build_mail_message`] is for an already-parsed
/// `mail_parser::Message`, which a `message/rfc822` attachment always is by
/// the time this module ever sees it.
fn parse_at_depth(
    bytes: &[u8],
    parent_key: &str,
    depth: u32,
    cumulative_expanded_bytes: &mut u64,
) -> Result<MailMessage, MailError> {
    let message = MessageParser::default()
        .parse(bytes)
        .ok_or(MailError::Unparseable)?;
    build_mail_message(&message, parent_key, depth, cumulative_expanded_bytes)
}

/// `cumulative_expanded_bytes` is shared, by mutable reference, across this
/// ENTIRE recursion tree — mail nesting AND any archive nesting reached
/// through an attachment — exactly the discipline
/// [`super::archive::expand_at_depth`]'s own doc states for its own
/// recursion (module doc, "Container recursion is one shared budget").
fn build_mail_message(
    message: &mail_parser::Message<'_>,
    parent_key: &str,
    depth: u32,
    cumulative_expanded_bytes: &mut u64,
) -> Result<MailMessage, MailError> {
    if let Some(kind) = sealed_kind(message) {
        return Err(MailError::Sealed(kind));
    }

    if let Some(detail) = degraded_body_part(message) {
        return Err(MailError::Degraded(detail));
    }

    let text_index = message.text_body.first().copied();
    let html_index = message.html_body.first().copied();
    let text_body = text_index
        .and_then(|_| message.body_text(0))
        .map(|c| c.into_owned());
    let html_body = if genuine_html(text_index, html_index) {
        message.body_html(0).map(|c| c.into_owned())
    } else {
        None
    };

    let mut references = header_ids(message.references());
    for id in header_ids(message.in_reply_to()) {
        if !references.contains(&id) {
            references.push(id);
        }
    }

    let attachment_parts: Vec<&MessagePart<'_>> = message.attachments().collect();
    let declared_count = attachment_parts.len();
    let (to_process, overflow) = if declared_count > MAX_ZIP_ENTRIES {
        (&attachment_parts[..MAX_ZIP_ENTRIES], true)
    } else {
        (&attachment_parts[..], false)
    };

    let mut coverage = Vec::new();
    if overflow {
        coverage.push(CoverageRow {
            path: None,
            status: Coverage::Unsupported,
            detail: Some(format!(
                "message declares {declared_count} attachments, exceeding the \
                 {MAX_ZIP_ENTRIES}-attachment MAX_ZIP_ENTRIES ceiling (reused from \
                 archive::MAX_ZIP_ENTRIES, R2); only the first {MAX_ZIP_ENTRIES} were \
                 considered, the remainder were never opened"
            )),
            bytes: None,
        });
    }

    let mut attachments = Vec::new();
    let mut seen_names: BTreeSet<String> = BTreeSet::new();
    let mut seen_collisions: BTreeMap<String, String> = BTreeMap::new();

    'attachments: for part in to_process {
        let is_message = part.is_message();
        let filename = part.attachment_name().unwrap_or_default().to_string();

        if filename.is_empty() {
            coverage.push(CoverageRow {
                path: None,
                status: Coverage::Error,
                detail: Some(
                    "an attachment with no recoverable name (neither Content-Disposition's \
                     filename nor Content-Type's name attribute) has no meaningful coordinate \
                     and is refused rather than silently keyed against its parent"
                        .to_string(),
                ),
                bytes: None,
            });
            continue;
        }
        if !filename.split('/').all(is_plain_name) {
            coverage.push(CoverageRow {
                path: Some(filename.clone()),
                status: Coverage::Excluded,
                detail: Some(
                    "attachment name is not path-safe (an absolute path, a `..` component, or \
                     a `\\`) per the same enclosed-path discipline archive.rs applies to a ZIP \
                     entry name"
                        .to_string(),
                ),
                bytes: None,
            });
            continue;
        }
        if !seen_names.insert(filename.clone()) {
            coverage.push(CoverageRow {
                path: Some(filename.clone()),
                status: Coverage::Excluded,
                detail: Some(format!(
                    "duplicate attachment name {filename:?}; the first occurrence was admitted, \
                     this later one sharing the identical name is refused rather than silently \
                     shadowing it"
                )),
                bytes: None,
            });
            continue;
        }
        let collision = archive::collision_key(&filename);
        if let Some(prior) = seen_collisions.get(&collision) {
            coverage.push(CoverageRow {
                path: Some(filename.clone()),
                status: Coverage::Excluded,
                detail: Some(format!(
                    "attachment {filename:?} collides with previously admitted attachment \
                     {prior:?} once both are Unicode-NFC-normalised and case-folded; refused \
                     rather than silently shadowed"
                )),
                bytes: None,
            });
            continue;
        }
        seen_collisions.insert(collision, filename.clone());

        let content = part.contents().to_vec();
        if content.len() as u64 > MAX_ENTRY_UNCOMPRESSED_BYTES {
            coverage.push(CoverageRow {
                path: Some(filename.clone()),
                status: Coverage::Unsupported,
                detail: Some(format!(
                    "attachment's decoded size exceeded the {MAX_ENTRY_UNCOMPRESSED_BYTES}-byte \
                     MAX_ENTRY_UNCOMPRESSED_BYTES ceiling (reused from archive.rs, R2) — a \
                     POST-decode admission check, not a pre-allocation stream bound (module \
                     doc, \"An honest gap\")"
                )),
                bytes: Some(content.len() as u64),
            });
            continue;
        }
        let entry_len = content.len() as u64;
        if cumulative_expanded_bytes.saturating_add(entry_len) > MAX_TOTAL_EXPANDED_BYTES {
            coverage.push(CoverageRow {
                path: Some(filename.clone()),
                status: Coverage::Unsupported,
                detail: Some(format!(
                    "expansion's whole-tree cumulative admitted size exceeded the \
                     {MAX_TOTAL_EXPANDED_BYTES}-byte MAX_TOTAL_EXPANDED_BYTES ceiling (reused \
                     from archive.rs, R2, shared across mail and archive recursion alike) at \
                     this attachment; remaining attachments at this level were never opened"
                )),
                bytes: Some(entry_len),
            });
            break 'attachments;
        }
        *cumulative_expanded_bytes += entry_len;

        let content_type = content_type_string(part);
        let hash = content_hash(&content);
        let is_archive = !is_message && archive::classify(&filename).1;
        let entry_adapter = if is_message || is_archive {
            None
        } else {
            archive::classify(&filename).0
        };
        let key_extractor = if is_message {
            MAIL_EXTRACTOR
        } else if is_archive {
            archive::ZIP_EXTRACTOR
        } else {
            entry_adapter.unwrap_or(UNSUPPORTED_CHILD_EXTRACTOR)
        };
        let key = child_key(parent_key, &filename, &hash, key_extractor);

        let mut nested_message = None;
        let mut nested_message_error = None;
        let mut nested_archive = None;

        if is_message {
            if depth + 1 > MAX_NESTING_DEPTH {
                nested_message_error = Some(format!(
                    "nested message at {filename:?} not opened: opening it would exceed the \
                     {MAX_NESTING_DEPTH}-level MAX_NESTING_DEPTH ceiling; the attachment itself \
                     is still admitted as a child resource, just not recursively parsed"
                ));
            } else {
                // `part.message()` is `mail-parser`'s OWN already-parsed
                // embedded value (module doc on `parse_at_depth`, and this
                // function's own doc): building from it directly, rather
                // than re-parsing `content` (`part.contents()` ==
                // `Message::raw_message()`) a second time, is what avoids
                // the boundary-adjacent-CRLF round-trip bug this module's
                // own tests caught.
                match part.message() {
                    Some(inner) => {
                        match build_mail_message(inner, &key, depth + 1, cumulative_expanded_bytes)
                        {
                            Ok(built) => nested_message = Some(Box::new(built)),
                            Err(error) => {
                                nested_message_error = Some(format!(
                                    "nested message at {filename:?} could not be parsed: {error}"
                                ));
                            }
                        }
                    }
                    None => {
                        nested_message_error = Some(format!(
                            "nested message at {filename:?}: mail-parser reported \
                             `is_message()` true but embedded no parsed message \
                             (unreachable in practice — recorded rather than panicking)"
                        ));
                    }
                }
            }
        } else if is_archive {
            if depth + 1 > MAX_NESTING_DEPTH {
                coverage.push(CoverageRow {
                    path: Some(filename.clone()),
                    status: Coverage::Unsupported,
                    detail: Some(format!(
                        "nested archive at {filename:?} not opened: opening it would exceed \
                         the {MAX_NESTING_DEPTH}-level MAX_NESTING_DEPTH ceiling; the \
                         attachment itself is still admitted as a child resource, just not \
                         recursively expanded"
                    )),
                    bytes: Some(entry_len),
                });
            } else {
                nested_archive = Some(Box::new(archive::expand_at_depth(
                    &content,
                    &key,
                    depth + 1,
                    cumulative_expanded_bytes,
                )));
            }
        }

        attachments.push(MailAttachment {
            filename,
            content_type,
            content,
            content_hash: hash,
            key,
            is_message,
            nested_message,
            nested_message_error,
            is_archive,
            nested_archive,
            entry_adapter,
        });
    }

    Ok(MailMessage {
        from: flatten_address(message.from()),
        to: flatten_address(message.to()),
        cc: flatten_address(message.cc()),
        sent: message.date().map(|d| d.to_rfc3339()),
        subject: message.subject().map(str::to_string),
        text_body,
        html_body,
        message_id: message.message_id().map(str::to_string),
        references,
        attachments,
        coverage,
    })
}

/// Caveat 2's own detection (module doc): the SAME [`mail_parser::MessagePartId`]
/// at the front of both `text_body` and `html_body` means the HTML is an
/// alias `mail-parser` synthesized, never a part the wire bytes declared
/// `Content-Type: text/html`. `(None, Some(_))` (HTML present, no text part
/// at all — an HTML-only message) is genuine: the synthesis code paths
/// (module doc) only ever fire from an EXISTING part shared into the
/// opposite vector, never fabricate one from nothing.
fn genuine_html(text_index: Option<u32>, html_index: Option<u32>) -> bool {
    match (text_index, html_index) {
        (Some(text), Some(html)) => text != html,
        (None, Some(_)) => true,
        _ => false,
    }
}

/// Caveat 1's own detection (module doc): a leaf, non-`message/rfc822`
/// attachment with no recoverable name.
fn degraded_body_part(message: &mail_parser::Message<'_>) -> Option<String> {
    message
        .attachments()
        .find(|part| !part.is_message() && part.attachment_name().is_none())
        .map(|part| {
            format!(
                "a body part became an unnamed attachment (mail-parser's own recovered shape \
                 for a message-like-but-broken input, e.g. an unterminated MIME boundary — \
                 SPIKE-G4.md gate (b)); content-type declared: {:?}",
                part.content_type().map(|ct| format!(
                    "{}/{}",
                    ct.ctype(),
                    ct.subtype().unwrap_or("*")
                ))
            )
        })
}

/// Module doc's "Sealed detection": structural only, from the message's own
/// top-level declared Content-Type, never by decrypting or verifying
/// anything.
fn sealed_kind(message: &mail_parser::Message<'_>) -> Option<&'static str> {
    if message.is_content_type("multipart", "encrypted") {
        return Some("multipart/encrypted, RFC 1847");
    }
    if message.is_content_type("application", "pkcs7-mime")
        || message.is_content_type("application", "x-pkcs7-mime")
    {
        return Some("application/pkcs7-mime, RFC 8551 S/MIME");
    }
    None
}

fn flatten_address(address: Option<&Address<'_>>) -> Vec<MailAddress> {
    let Some(address) = address else {
        return Vec::new();
    };
    address
        .iter()
        .map(|addr| MailAddress {
            name: addr.name().map(str::to_string),
            address: addr.address().map(str::to_string),
        })
        .collect()
}

/// `References:`/`In-Reply-To:` are always `Text` (one id) or `TextList`
/// (several) per RFC 5322 — never an address list — so this covers both
/// shapes plus the `Empty` case a header that is absent parses to.
fn header_ids(value: &HeaderValue<'_>) -> Vec<String> {
    match value {
        HeaderValue::Text(id) => vec![id.to_string()],
        HeaderValue::TextList(ids) => ids.iter().map(|id| id.to_string()).collect(),
        _ => Vec::new(),
    }
}

fn content_type_string(part: &MessagePart<'_>) -> Option<String> {
    let content_type = part.content_type()?;
    Some(match content_type.subtype() {
        Some(subtype) => format!("{}/{subtype}", content_type.ctype()),
        None => content_type.ctype().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> Vec<u8> {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/mail_corpus")
            .join(name);
        std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    // ------------------------------------------------------------------- F6

    #[test]
    fn parsing_is_a_pure_function_of_its_input() {
        let bytes = fixture("01-plain-text.eml");
        assert_eq!(
            parse_message(&bytes, "parent"),
            parse_message(&bytes, "parent")
        );
    }

    #[test]
    fn extractor_for_claims_only_eml() {
        assert_eq!(extractor_for("message.eml"), Some(MAIL_EXTRACTOR));
        assert_eq!(extractor_for("MESSAGE.EML"), Some(MAIL_EXTRACTOR));
        for other in ["message.msg", "report.docx", "archive.zip", "plain.txt"] {
            assert_eq!(extractor_for(other), None, "{other:?} must not be claimed");
        }
    }

    // --------------------------------------------- gate (b), fixture by fixture

    #[test]
    fn fixture_01_plain_text_message_shape() {
        let bytes = fixture("01-plain-text.eml");
        let message = parse_message(&bytes, "parent-key").expect("parses");
        assert_eq!(message.to.len(), 1, "manifest to_address_count");
        assert_eq!(message.cc.len(), 0, "manifest cc_address_count");
        assert!(message.text_body.is_some());
        assert_eq!(
            message.html_body, None,
            "no genuine HTML in the wire bytes — a synthesized alias must read as absent \
             (caveat 2)"
        );
        assert_eq!(message.subject.as_deref(), Some("Plain text status update"));
        assert!(message.attachments.is_empty());
    }

    #[test]
    fn fixture_02_multipart_alternative_both_bodies_are_genuine() {
        let bytes = fixture("02-multipart-alternative.eml");
        let message = parse_message(&bytes, "parent-key").expect("parses");
        assert_eq!(message.to.len(), 2, "manifest to_address_count");
        assert_eq!(message.cc.len(), 1, "manifest cc_address_count");
        assert!(message.text_body.is_some());
        assert!(
            message.html_body.is_some(),
            "a real multipart/alternative HTML part must NOT be treated as synthesized"
        );
        assert_eq!(message.subject.as_deref(), Some("Alternative body demo"));
    }

    #[test]
    fn fixture_03_with_attachment() {
        let bytes = fixture("03-with-attachment.eml");
        let message = parse_message(&bytes, "parent-key").expect("parses");
        assert_eq!(message.attachments.len(), 1, "manifest attachment_count");
        let attachment = &message.attachments[0];
        assert_eq!(attachment.filename, "report.txt");
        assert_eq!(
            attachment.content, b"Report body line one.\nReport body line two.",
            "manifest attachment_decoded_byte_len / decoded_content"
        );
        assert_eq!(attachment.content.len(), 43);
        assert!(!attachment.is_message);
        assert!(!attachment.is_archive);
        assert_eq!(
            attachment.entry_adapter,
            Some(super::super::text::TEXT_EXTRACTOR),
            "report.txt's own downstream extractor is text.rs's routing table, reused (R2) via \
             archive::classify — never UNSUPPORTED_CHILD_EXTRACTOR, which is only the \
             placeholder for an extension nothing claims"
        );
        assert_eq!(
            attachment.key,
            child_key(
                "parent-key",
                "report.txt",
                &content_hash(&attachment.content),
                super::super::text::TEXT_EXTRACTOR
            ),
            "F7/G9 child key composition"
        );
    }

    #[test]
    fn fixture_04_nested_rfc822_recurses_via_the_same_function() {
        let bytes = fixture("04-nested-rfc822.eml");
        let message = parse_message(&bytes, "parent-key").expect("parses");
        assert_eq!(message.attachments.len(), 1, "manifest attachment_count");
        let attachment = &message.attachments[0];
        assert!(attachment.is_message, "manifest nested_message_count");
        assert_eq!(attachment.filename, "original.eml");
        let nested = attachment
            .nested_message
            .as_deref()
            .expect("nested message parses within the depth ceiling");
        assert_eq!(nested.subject.as_deref(), Some("Original note"));
        assert_eq!(
            nested.text_body.as_deref(),
            Some("This is the original message text, now nested one level deep.\r\n"),
            "the fixture's own bytes carry a blank line before the boundary; RFC 2046 5.1.1 \
             consumes only the LAST CRLF as the boundary delimiter, so the other is real body \
             content — VERIFIED independently against Python's stdlib email package \
             (manifest.json's own body_text_decoded_correction_note carries the full argument)"
        );
        assert_eq!(
            nested.attachments.len(),
            0,
            "the nested message itself has no attachments"
        );
        assert_eq!(
            attachment.key,
            child_key(
                "parent-key",
                "original.eml",
                &attachment.content_hash,
                MAIL_EXTRACTOR
            ),
            "a nested message's own key uses MAIL_EXTRACTOR, not the container's identity \
             (archive.rs's own warning against recording every child as adapter=container)"
        );
    }

    #[test]
    fn fixture_05_encoding_zoo_round_trips_byte_correct() {
        let bytes = fixture("05-encoding-zoo.eml");
        let message = parse_message(&bytes, "parent-key").expect("parses");
        assert_eq!(
            message.subject.as_deref(),
            Some("Café update ☕"),
            "RFC 2047 UTF-8 B-encoding"
        );
        assert_eq!(
            message.from[0].name.as_deref(),
            Some("René Dupont"),
            "RFC 2047 ISO-8859-1 Q-encoding"
        );
        assert_eq!(
            message.text_body.as_deref(),
            Some("Prix unitaire: 12€ le café.\nTotal: 24€."),
            "windows-1252 quoted-printable round-trip"
        );
        assert_eq!(message.attachments.len(), 1);
        assert_eq!(message.attachments[0].filename, "blob.bin");
        assert_eq!(message.attachments[0].content.len(), 8);
        assert_eq!(
            message.attachments[0].content_type.as_deref(),
            Some("application/octet-stream")
        );
    }

    /// **Gate (b)'s own pass criterion, fixture 06**: a message with zero
    /// RFC 5322 header lines must fail outright.
    #[test]
    fn fixture_06_malformed_no_headers_is_refused() {
        let bytes = fixture("06-malformed-no-headers.eml");
        let error = parse_message(&bytes, "parent-key").expect_err("must refuse, never partial");
        assert_eq!(error, MailError::Unparseable);
    }

    /// The spike's own caveat-1 evidence, made a real regression pin: the
    /// unterminated-boundary fixture must be refused as [`MailError::Degraded`],
    /// never silently accepted with a downgraded body.
    #[test]
    fn diagnostic_broken_mime_is_refused_as_degraded_not_silently_accepted() {
        let bytes = fixture("diagnostic-not-manifest-broken-mime.eml");
        let error = parse_message(&bytes, "parent-key").expect_err("must refuse, never partial");
        assert!(
            matches!(error, MailError::Degraded(_)),
            "must be classified Degraded: {error:?}"
        );
    }

    // ------------------------------------------------------- caveat 2, directly

    #[test]
    fn genuine_html_rejects_a_shared_index_and_admits_a_distinct_one() {
        assert!(
            !genuine_html(Some(3), Some(3)),
            "the same part index in both vectors is a synthesized alias"
        );
        assert!(
            genuine_html(Some(1), Some(2)),
            "distinct indices are two real, separately-declared parts"
        );
        assert!(
            genuine_html(None, Some(2)),
            "HTML with no text part at all is a genuine HTML-only message"
        );
        assert!(!genuine_html(Some(1), None), "no HTML at all");
        assert!(!genuine_html(None, None), "no body at all");
    }

    // --------------------------------------------------------- admission discipline

    #[test]
    fn an_attachment_with_a_traversal_name_is_refused() {
        let raw = b"From: a@example.com\r\nTo: b@example.com\r\nSubject: x\r\nDate: Mon, 1 Jan 2024 00:00:00 +0000\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary=\"B\"\r\n\r\n--B\r\nContent-Type: text/plain\r\n\r\nbody\r\n--B\r\nContent-Type: application/octet-stream; name=\"../../etc/passwd\"\r\nContent-Disposition: attachment; filename=\"../../etc/passwd\"\r\nContent-Transfer-Encoding: base64\r\n\r\nAAAA\r\n--B--\r\n";
        let message = parse_message(raw, "parent-key").expect("outer message parses");
        assert!(
            message.attachments.is_empty(),
            "a traversal-shaped name must never be admitted: {:?}",
            message.attachments
        );
        let row = message
            .coverage
            .iter()
            .find(|r| r.path.as_deref() == Some("../../etc/passwd"))
            .expect("a coverage row names the refusal");
        assert_eq!(row.status, Coverage::Excluded);
    }

    #[test]
    fn a_sealed_smime_message_gets_its_own_honest_status() {
        let raw = b"From: a@example.com\r\nTo: b@example.com\r\nSubject: sealed\r\nDate: Mon, 1 Jan 2024 00:00:00 +0000\r\nMIME-Version: 1.0\r\nContent-Type: application/pkcs7-mime; smime-type=enveloped-data\r\nContent-Transfer-Encoding: base64\r\n\r\nAAAA\r\n";
        let error = parse_message(raw, "parent-key").expect_err("must refuse as sealed");
        assert!(matches!(error, MailError::Sealed(_)), "{error:?}");
    }
}
