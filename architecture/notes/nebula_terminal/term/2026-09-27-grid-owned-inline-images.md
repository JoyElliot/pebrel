# Grid-owned inline images

## Status

Proposed; implemented locally and verified on Windows. Cross-platform CI has
not run for this change.

## Context

OSC 1337 images previously used an absolute history-row anchor outside the
terminal grid. The stream observer decoded dimensions but discarded the
requested width/height, read the cursor before synchronized VTE replay, and
injected extra newlines. OMP saves its cursor, moves into reserved image rows,
emits the image, then restores its cursor; those extra newlines scrolled the
reserved rows and drew the image over unrelated UI.

An absolute row anchor also survives erasure and alternate-screen switches.
An asynchronous decode can complete after the content that requested it has
already been removed.

## Evidence

- [`event_loop.rs`](../../../../nebula_terminal/src/event_loop.rs) observes OSC
  sequences in addition to VTE parsing; DEC 2026 can defer VTE cursor updates.
- [`inline_image.rs`](../../../../nebula_terminal/src/inline_image.rs) calculates
  requested cell, pixel and percentage sizes against the available viewport.
- The OSC 1337 specification defines those
  units and aspect-ratio behavior. Its reference
  image insertion helper (source links are retained in the PR evidence)
  clamps both dimensions at the right margin, advances between image rows,
  and leaves the final newline to the application.
- [`tests/inline_image.rs`](../../../../nebula_terminal/tests/inline_image.rs)
  exercises the OMP save/move/image/restore sequence, every stream split point,
  timeout replay, erasure, alternate screens, resizing and title isolation.

## Decision

The core owns image geometry and lifetime. Each occupied grid cell holds an
image identity and source tile coordinate in its optional extra storage. Grid
edits move or erase image tiles just as they do text. Render snapshots contain
only IDs and contiguous visible fragments; no strong image owner escapes the
terminal lock. Renderer queues and caches hold weak identities, so a completed
decode cannot resurrect an image after its last cell disappears.

VTE 0.15 has no extension OSC callback. The stream processor inserts a private,
one-shot, randomly keyed title token immediately after the image sequence.
VTE replays this token in order with cursor commands, including DEC 2026.
The title handler consumes an exact queued token before normal title effects.
Pending jobs and encoded bytes are bounded. The stream owns pending payloads;
the Term holds weak replay entries so disconnecting releases an unfinished
synchronized batch even when its pane is retained. The image is inserted only at
that replay point; the token is never exposed as an application title.

Cached rows outside the logical grid must release transient image owners even
when their text allocation is retained for reuse. A `GridCell::discard` hook
releases only transient metadata; it does not reset cached text or inflate the
hot `Cell` size. Both renderers clip fragments against the full image bounds.

## Rejected alternatives

- Force `PI_FORCE_IMAGE_PROTOCOL` globally: this is an OMP capability-detection
  concern and would override user choices and leak into nested sessions.
- Keep absolute anchors and clear a renderer cache on selected events: partial
  erasure, reflow and late decode would still require a second grid model.
- Flush synchronized output on image arrival: it exposes intermediate frames
  and makes placement depend on stream chunk boundaries.
- Patch ConPTY flags: the release runtime already transports the tested OMP
  sequence. Tests must include the same adjacent runtime as the release.
- Encode texture crops as separately rounded source rectangles: tiny images
  enlarged to multiple rows can lose fragments through pixel rounding. Keep
  the full texture transform and clip destination fragments instead.

## Consequences

Images allocate optional per-cell metadata and produce at most one draw
fragment per contiguous image span per row. A single image is limited to
16,384 occupied cells in addition to existing encoded/decoded-byte limits.
An image at the one-past-right-margin position is rejected rather than
overwriting the final text cell; applications can move the cursor explicitly.

The existing VTE synchronized-output size/timeout bounds still apply. OSC 133
prompt metadata remains a pre-existing out-of-band observer and is not made
transactional by this change. Legacy rendering is compile-checked; live visual
acceptance here covers the Windows GPUI shell and its bundled ConPTY runtime.

This change does not advertise a new image capability. OMP's environment-based
OSC 1337 selection still does not recognize Pebrel; its separate dynamic Sixel
probe is outside this existing-protocol repair.

## Validation

The protocol/lifetime integration tests and GPUI decode/cache tests cover the
changed contracts. Windows Computer Use verified requested widths, column
anchors, screen clearing, alternate-screen hide/restore, and the complete OMP
18.3.3 client rendering a prepared image tool result without a model call.
The source tree also passes the architecture budget/dependency checker.

A local Windows x64 Rust 1.97.1 release harness compared 50,000 CRLF text
lines at 120 columns by 40 rows, with 10,000 history rows, against base
`7b0cedb5`. Seven sequential plus seven alternating samples had median feed
times of 41,494 us (base) and 42,250 us (patch), a 1.82% local increase with
overlapping sample ranges. Median snapshot time increased from 153.5 to
160 us. `Cell` remained 24 bytes; optional `CellExtra` grew from 40 to 64
bytes. These figures describe that workload and build only, not a zero-cost
claim, application frame rate or cross-platform throughput.

## Supersedes

None.

## Revisit when

VTE provides ordered extension OSC callbacks, another image protocol is added,
or representative workloads show per-cell metadata or row-fragment drawing
to be a material cost. Prefer the parser callback over the private title token
when that interface exists and retains synchronized-output ordering.
