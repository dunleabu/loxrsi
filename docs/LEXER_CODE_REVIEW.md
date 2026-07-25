# `src/lexer.rs` review — robustness, readability, performance

Reviewed at the state where `State::flush` was introduced. All 49 tests pass.
Everything below was checked against a compiled copy of the file, not read off
by eye; measurements and method are in the last section.

Overall this is a clean, well-tested lexer. The state machine is coherent, the
recent EOF fixes hold up under adversarial input, and the test suite is better
than most hand-written lexers get. The findings are one real input-handling bug,
one re-opened maintenance hazard, a handful of diagnostic-quality issues, and a
performance profile with about 2x of easily reachable headroom.

---

## Robustness

### 1. CRLF input fails to lex at all — `from_start:313-315`

`\r` isn't in the whitespace arms, so it falls to the `bad character` arm:

```
lex("a\r\nb")  =>  Err[ Error("bad character: '\r'") ]
```

Any file saved with Windows line endings, or pasted from a Windows editor, fails
wholesale — and the message points at an invisible character, so the report is
close to unactionable. Add `'\r'` to the whitespace arms. Note the arms are also
worth collapsing to `' ' | '\t' | '\n' | '\r' => whitespace(text)`.

If you want `\r\n` to count as one line rather than relying on the `\n`, that's
a `step` change, but simply treating `\r` as whitespace is correct today because
`line_num` is driven by the `\n` that follows.

### 2. `State::flush`'s `_ => None` re-opens the hazard the exhaustive match closed

`State::flush` (99-105) is a genuine readability improvement over the inline
`match` in `lex` — the flush concern now lives on the type that owns it. But the
catch-all arm gives up the property that made the previous version safe:

```rust
_ => None,
```

Add a `State` variant that holds a pending token and this compiles silently,
dropping that token at end of input — which is exactly the bug fixed in
`8987800`. Listing the two inert states explicitly costs nothing and turns that
class of mistake into a compile error:

```rust
Self::Start | Self::InComment => None,
```

Same reasoning as the duplicated-fallback discussion: the goal isn't to be
correct now, it's to make the next edit unable to be wrong quietly.

### 3. Panics on a `Result`-returning path — `to_number:219-224, 234-237`

Three panic sites: `strip_suffix('.').unwrap()`, and two
`expect("failed number conversion")`.

I could not reach any of them. `strip_suffix` is safe because at that point the
slice always ends at the dot, at EOF or not, and both `parse::<f64>` calls only
ever see `digits` or `digits.digits` (very long inputs saturate to `inf`, they
don't error). So this is not a live bug.

It is still the wrong failure mode for the module. `lex` returns
`Result<_, Vec<TokenContext>>` and models every other malformed input as
`Token::Error` — a bad number should join them rather than abort the process.
The invariant protecting the `unwrap` is real but unstated, and it is exactly
what a future change would break: hex literals, `1e10` exponents, or a leading-`+`
tweak all invalidate "the slice is digits and at most one dot" without touching
these lines. Returning `Token::Error` here costs three lines and removes the
category.

### 4. Numbers are ASCII-only but identifiers are all-Unicode — `from_start:337-338`

`_ if c.is_ascii_digit() => to_number` versus `_ if c.is_alphanumeric() =>
to_identifier`, with `okay_for_id` also using `is_alphanumeric`. The pairing is
inconsistent, and non-ASCII digits fall through the number arm into the
identifier arm:

```
lex("١٢٣")  =>  [Identifier("١٢٣")]   // Arabic-Indic digits
lex("१२३")  =>  [Identifier("१२३")]   // Devanagari digits
```

`char::is_alphanumeric` is `Alphabetic | Nd | Nl | No`, so every Unicode digit
category is an identifier start here. Lox specifies ASCII identifiers, so
`is_ascii_alphanumeric` in both places would match the spec and remove the
oddity. If you'd rather keep Unicode identifiers as a deliberate extension —
they work correctly, `lex("café")` is fine — then exclude the numeric categories
from the identifier predicate so digits and identifiers can't overlap. Either
way it should be a decision recorded in a comment; right now it reads as an
oversight.

### 5. Diagnostics: positions point past the token, not at it

`add_context` (80-88) reads `char_pos` at the moment it's called, which is after
the token's characters have been consumed:

```
"!"      =>  Bang@1:1        // correct, by accident: step at EOF doesn't advance char_pos
"! "     =>  Bang@1:2        // same token, different column
"  @"    =>  Error@1:3       // the '@' is at column 3, reported as 3 — correct here
"a\r\nb" =>  Error@2:0       // the '\r' is at line 1; reported at line 2, column 0
```

Three separate problems compound:

- **Multi-char tokens report their end.** `Identifier("café")` in `"café;"`
  reports the semicolon's column, not the identifier's start.
- **`char_pos = 0` for `\n`** (`step:50`), so a token whose scan crosses a
  newline reports column 0, and a `\r` immediately before a newline is
  attributed to the *next* line, as above.
- **EOF and non-EOF disagree** for the same token, since `step` doesn't advance
  `char_pos` past the end.

The fix is to capture `(line_num, char_pos)` at token start — `mark()` is the
natural place, since it already marks the start — and have `add_context` use the
saved pair. That makes all three symptoms go away together.

This matters more than it looks: positions are the one part of a lexer's output
users see directly, and **no test asserts a single `Context` value**. The whole
subsystem is unverified, which is why all of the above went unnoticed.

### 6. `Token::Error` in the token enum, and tokens discarded on failure

Two related design points, both defensible but worth revisiting before the
parser grows:

- `Token::Error(String)` means every downstream `match` on `Token` carries an
  arm for a variant that can never appear in the `Ok` path, since `lex`
  partitions errors out. A separate `LexError { message, context }` type would
  let the compiler enforce what the partition already guarantees.
- The `Err` branch returns *only* errors, discarding every valid token. That
  forecloses parser-side error recovery — "parse what you can, report the rest"
  — which is normally the reason a lexer collects multiple errors instead of
  bailing on the first. If you keep the current shape, `has_error` is redundant
  with `!errors.is_empty()` and can go.

### 7. Small things

- **`impl<'a> TextInput<'_>`** (14): the declared `'a` is used only by `new`'s
  signature; the self type is over an elided anonymous lifetime. It compiles and
  behaves, but `impl<'a> TextInput<'a>` is what's meant and is what a reader
  expects.
- **`mark()` returns a `usize`** (39-42) that no caller uses.
- **`pub current`** (8) on a private struct — the `pub` has no effect.
- **`pr`** (68-78) is the module's only compiler warning, and its `-> ()` is
  redundant. Either `#[allow(dead_code)]` it as a deliberate debug aid or drop
  it. It is at least now panic-safe on empty input, which it wasn't before the
  `slice` fix.
- **Strings have no escape sequences**, so `"a\"b"` can't be written. That
  matches jlox's scanner, so it's correct for Lox — worth a one-line comment in
  `to_string` so it reads as intentional rather than missing.
- **Unterminated strings report the EOF position**, not the opening quote's.
  For a runaway quote in a long file, the opening position is the useful one.

---

## Readability

The dispatch design reads well: `StepOut`, one function per token shape, and
free functions over `&mut TextInput` rather than methods keeps each scanner
independently readable. `to_number` / `to_identifier` / `to_string` are each
easy to follow in isolation, and the test names document the intended semantics
better than comments would.

### The machine is a hybrid, and the states are the awkward half

`from_start` is a maximal-munch scanner: `to_identifier` and `to_string`
internally consume many characters within a single "step". `State` exists only
for the three cases that couldn't be done inline — one-char lookahead
(`OnLookahead`), comment skipping (`InComment`), and the pushed-back dot
(`AddDot`). So there are two different mechanisms for the same job, and the
per-character `lex` loop is a consequence of the weaker one.

The root cause is that `TextInput` can't peek. `CharIndices` is not `Peekable`
and no next-char is cached, so `to_number` can't ask "is the char after the dot
a digit?" without consuming the dot — which is precisely why `AddDot` exists,
and why `!` has to be handled across two turns. Give `TextInput` a `peek`
(either `Peekable<CharIndices>` or a stored lookahead char) and:

- `AddDot` disappears — `to_number` just doesn't consume a dot not followed by
  a digit.
- `OnLookahead` disappears — `if_equal` becomes a peek at the point of decision.
- `InComment` disappears — comment skipping becomes a loop inside `maybe_comment`,
  like the other multi-char scanners.
- `State` disappears entirely, `flush` with it, and `lex` becomes
  `while let Some(t) = next_token(&mut txt) { … }`.

That removes the whole class of "state pending at EOF" bug — the one that
produced both of the last two commits — by construction rather than by handling.
It is the single largest simplification available here, and it also fixes the
performance profile discussed below. It's a real refactor, not a cleanup, so
it's a judgement call whether to do it now or after the parser settles; I'd do
it before adding any more token types.

### Commented-out code

Lines 157 (`//EOF`), 227-230 (the previous unterminated-number error), 326
(`//'/' => to_start(...)`), 380 (`//txt.pr()`), 397, 403, 407 (debug
`println!`s). Git holds all of it. The block inside `to_number`'s return is the
worst of them — it sits between the `State::AddDot` and the token it returns,
which is the exact spot where a reader is trying to understand the trickiest
transition in the file.

### Naming and argument order

- `if_equal(text, double, single)` at `step:372` passes the pair in the reverse
  of the field order in `State::OnLookahead { single, double }`. Both arguments
  are `Token`, so a transposition typechecks. Ordering the parameters
  `(single, double)` to match the struct, or naming them `if_next_is_equal`,
  removes the trap.
- `to_start` means "emit and return to Start" while `to_number` means "scan a
  number". Same prefix, two different meanings. `emit` / `emit_token` would
  separate them.
- `okay_for_id(Option<char>)` taking an `Option` is unusual — a
  `is_id_char(char)` predicate plus `map_or(false, is_id_char)` at the one call
  site reads more directly, and the function becomes reusable.

### Tests

Strong: the EOF corner cases, the keyword-prefix cases, the trailing-dot family
and the non-ASCII cases all pin down behaviour that was recently wrong or is
easy to get wrong. `lookahead_operators_agree_at_eof_and_mid_input` tests a
property rather than an instance, which is the right instinct.

Gaps, in priority order:

1. **No `Context` assertions anywhere** — see robustness §5.
2. **No CRLF test** — would have caught §1.
3. Multi-line string spanning a newline (works: `"multi\nline"` → line 2).
4. Multiple errors in one input — `lex` collects a `Vec` but only single-error
   cases are tested.
5. Unicode digits (§4), to lock in whatever you decide.

---

## Performance

Method: `rustc 1.93.1 -O`, i5-7500T, a 1.82 MB synthetic Lox source (320k
tokens) of `var`/`if`/`print`/string/number/comment lines, best of 9 in-process
runs, repeated 3 times per variant. Run-to-run spread on a single timing is
±10%, so only reproducible non-overlapping deltas are reported below. Each
variant passes the full 49-test suite except the last, which is an ablation and
deliberately produces wrong output.

| Variant | MB/s | vs. baseline |
|---|---|---|
| Current file | 111, 114, 107 | — |
| `State` shrunk to 4 bytes | 128, 129, 130 | **+16%** |
| … + `Vec::with_capacity(len/8)` | 119, 120, 117 | *slower than above* |
| Dead newline `mark()` removed (alone) | 119 | **+7%** |
| … + no `String` allocation (ablation) | 176, 171, 168 | **+54%** |

At ~110 MB/s this is fast enough that lexing will not be your bottleneck for
any plausible Lox program. The items below matter if you want the lexer to be
respectable on its own terms.

### 1. `String` allocation per identifier and string literal — the dominant cost

`Token::Identifier(String)` and `Token::String(String)` mean one heap allocation
plus a copy for every identifier occurrence and every string literal. Ablating
just those allocations (keeping everything else, including the `to_owned` call
sites) is worth **+44%** over the otherwise-identical variant. It's the only
change here with a large, unambiguous effect.

The idiomatic fix is zero-copy: `Identifier(&'a str)` borrowing from the source,
since `TextInput` already holds `data: &'a str` and `slice()` already produces
exactly the right subslice. That threads a lifetime through `Token`,
`TokenContext` and `lex`'s signature, which is a mechanical but wide change, and
it constrains the parser to borrow from the source text for as long as the AST
lives. The alternative — interning identifiers into a `Vec<String>` and storing
a `u32` symbol id — is what most real implementations do, keeps `Token` `Copy`,
and pays off again in the resolver. Either is a deliberate architectural choice;
I'd decide it before the parser depends on `Token: 'static`.

Secondary benefit: `Token` is 32 bytes today, driven entirely by `String` (24
bytes). `TokenContext` is 48. With a `u32` symbol id, `Token` drops to 16 and
`TokenContext` to 32, halving the memory traffic of building the output `Vec`.

### 2. `State` carries two `Token`s — 64 bytes, and a 112-byte `StepOut`

This one is my doing, from the last refactor. Measured sizes:

```
Token 32   TokenContext 48   State 64   StepOut 112   TextInput 80
```

`OnLookahead { single, double }` stores two 32-byte `Token`s, so `State` went
from a 1-byte tag to 64 bytes, and `StepOut = (State, Option<TokenContext>)` —
returned by value from every scanner, on every character — is 112 bytes.

Storing the operator char instead and mapping it to the token pair in one
function recovers this: `State` → 4 bytes, `StepOut` → 56, worth a reproducible
**+16%**. Critically it keeps the property the refactor was for, because the
pairing is still stated exactly once:

```rust
enum State { Start, OnLookahead(char), InComment, AddDot }

/// The (single, double) token pair for each char that may start a two-char
/// operator. The one place this pairing is stated.
fn lookahead_tokens(c: char) -> (Token, Token) {
    match c {
        '!' => (Token::Bang, Token::BangEqual),
        '=' => (Token::Equal, Token::EqualEqual),
        '>' => (Token::Greater, Token::GreaterEqual),
        '<' => (Token::Less, Token::LessEqual),
        _ => unreachable!("not a lookahead char: {c}"),
    }
}
```

`from_start` then collapses to `'!' | '=' | '>' | '<' => on_lookahead(text, c)`,
and both `step` and `flush` call `lookahead_tokens`. I verified this variant
passes all 49 tests. The `unreachable!` is the one wart — it trades a
compile-time-total mapping for a runtime-partial one, which is a fair objection.
Note that the peek-based refactor in the readability section deletes `State`
outright and so subsumes this entirely; if you're doing that, skip this.

### 3. Dead `mark()` on every newline — `lex:386-388`

```rust
if c == '\n' {
    txt.mark();
}
```

Nothing reads `left` before the next `mark()`: `to_number`, `to_identifier` and
`to_string` each call `mark()` themselves as their first act, and no other path
calls `slice()`. Removing it leaves all 49 tests passing and is worth **+7%** —
it's a branch on every character plus a store on every line, for nothing. (A
companion `if c == 'o' { txt.mark(); }` was already removed; this is the last of
that pair.)

### 4. Don't add `Vec::with_capacity` — measured negative

The obvious "avoid reallocation" tweak made things **worse** (129 → 119 MB/s).
`len/8` under-estimates real token density here (~1 per 5.7 bytes) so it still
reallocates, while paying a large upfront allocation and first-touch page
faults. If you want to pre-size, measure the ratio on real input first; the
default growth strategy is already decent for 48-byte elements. Listed here
mainly so it doesn't get "optimised" in later.

### 5. Structural ceiling: per-char `CharIndices` decoding and call-per-character dispatch

The remaining cost is the shape of the loop: every character goes through
`CharIndices::next` (UTF-8 decode), then a function call returning a
multi-word tuple, then a `match` in `lex` to sort the result. Production lexers
scan `&[u8]` with a byte-level dispatch table and only decode UTF-8 inside
string literals and identifiers, which is where the order-of-magnitude
difference between ~100 MB/s and ~1 GB/s lives.

I would not do that here. It costs the readability that is currently this
module's main strength, for a speedup that no Lox program will notice. The
peek-based refactor is the version of this worth having: it removes the
per-character state dispatch as a *side effect* of making the code simpler,
since each scanner then runs its own tight loop and `lex` iterates per token
rather than per character.

### Not a problem

The keyword lookup (`to_identifier:286-304`) matching on `text.slice()` is fine.
`rustc` compiles a `&str` match into a length switch plus short comparison
chains, it allocates nothing on the keyword path, and `to_owned()` is called
only in the identifier fallback arm. A `phf`-style perfect hash would not be
measurable at 16 keywords.

---

## Suggested order

1. `'\r'` as whitespace (§1) — a real bug affecting real files, one line.
2. `Self::Start | Self::InComment => None` in `flush` (§2) — one line, closes a
   bug class.
3. Capture position at token start; add `Context` assertions to the tests (§5) —
   the largest *correctness* gap that tests currently can't see.
4. Delete the dead newline `mark()` (perf §3) and the commented-out code — pure
   subtraction, +7%.
5. Decide the Unicode identifier question and write it down (§4).
6. Replace the three number-path panics with `Token::Error` (§3).
7. Then choose one of: peek-based refactor (readability, removes `State`, and
   subsumes perf §2), or `State::OnLookahead(char)` if you want the +16% without
   the larger change.
8. Zero-copy or interned identifiers (perf §1) — the only remaining large
   performance lever, and best decided together with the parser's needs.
