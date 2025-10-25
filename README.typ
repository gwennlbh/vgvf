= `vgv`: Vector Graphics Video Format

An experimental video format for vector graphics.

Magic bytes `vgv1\n` then `\n`-separated frames

== Tracks

=== Audio

=== Video

== Frames

=== `I`initialization frame

Sets attributes on the SVG tag, such as width, height, viewBox, etc.

Special attributes that affect VGV but are _not_ set on the SVG tag:

/ d="ms": duration of each frame in milliseconds

`I` then space-separated list of `key="value"` pairs.

=== `S`tyle frames

`S` then minified CSS rules. 

The rules get added to a global stylesheet, which is applied to all subsequent frames. Thanks to CSS's last-rule-wins behavior, later rules can override earlier ones (if one is careful about specificity).

Players MAY choose to compress the global stylesheet to prevent it from growing very large, especially if many CSS frames are used to achieve a flickering effect, for example.

=== `A`udio frames

`A` then `WAV` (for the format, others may be added later), then raw audio bytes.

=== `F`ull frames

`F` then SVG content.

=== `D`iff frames

`D` then a delta encoding of the SVG content, relative to the previous frame. Deltas are Tab-separated chunks:

/ Added: `+` then the added text
/ Deleted: `-` then the number of characters to delete
/ Unchanged: `=` then the number of unchanged characters

See diff-match-patch-rs (https://docs.rs/diff-match-patch-rs/0.5.1/diff_match_patch_rs/)'s `diff_to_delta` (in `Compat` mode)

== Variants

=== Text

=== Binary

=== HTML-Encapsulated
