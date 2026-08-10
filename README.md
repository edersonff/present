# present

> Terminal picker for humans and JSON ask/answer protocol for agents. One binary, two faces.

When an agent hits a fork — pick a module, pick a thumbnail style, pick a provider — it used to
print "which one?" to stdout and try to read the human's reply out of its own conversation log.
That is fragile. `present` does that one job: it takes a message and a list, asks the human, and
returns the pick as a clean value on stdout.

When a person types `present` in a terminal, they get a fuzzy-filterable list of everything on the
sheol shelf. They pick one, fill the arguments, and it runs.

## Install

```
cargo build --release && install -m755 target/release/present ~/.local/bin/present
```

## The two faces

**Agent face** — `--ask` with flags, or `--json` with stdin. The response is always one line.

```
$ PRESENT_AUTO_PICK=1 present --ask "pick a thumbnail" --options '["bold","minimal","photo"]' --json
{"choice":"bold"}
```

**Human face** — no args in a terminal. A ratatui list of shelf modules with a live fuzzy filter.
Pick one, fill the arguments, it runs.

```
$ present
```

## Input

Three shapes:

- Flags — `present --ask "<message>" --options '["a","b","c"]' [--json] [--interactive]`
- JSON on stdin — `present --json`, reads `{"message":"<words>","options":["a","b","c"]}`
- Browse — `present` (no args, in a terminal), or `present --module <name>` to skip to one module

## Output

Plain mode (no `--json`):

- picked: the option on stdout, exit `0`
- cancelled: empty stdout, exit `2`
- bad input: message on stderr, exit `1`

JSON mode (`--json`):

- picked: `{"choice":"a"}`, exit `0`
- cancelled: `{"choice":null,"cancelled":true}`, exit `2`
- bad input: `{"error":"...","code":"bad-input"}`, exit `1`

## The agent contract

```
choice=$(printf '%s' "$request_json" | present --json)
```

Or with flags (no stdin needed):

```
choice=$(PRESENT_AUTO_PICK=1 present --ask "which provider" --options '["openai","anthropic"]' --json)
```

`PRESENT_AUTO_PICK=1` auto-selects the first option without reading `/dev/tty`. It exists for
non-interactive environments (CI, agents with no attached terminal). Don't set it when a human is
present — they would never see the prompt. `PRESENT_AUTO_PICK=0` or `false` disables it.

## Browse mode

`present` with no args in a terminal:

1. If the current directory has a `sheol.json`, present reads it and shows the param form for that
   module directly.
2. If not, present calls `sheol search --json` and shows a ratatui list of every module on the
   shelf. Type to fuzzy-filter by name, description, id, or tags. Arrows move, enter picks.
3. After picking, present parses the module's `entry.cli`, shows each argument as a field with its
   default value, and lets you override each one. Press enter on an empty field to keep the default.
4. On submit, the filled command runs.

`present --module <name>` skips the list and jumps straight to the param form for that module.

## Progress strip

A subcommand reads `{current,total,label}` JSON lines from stdin and renders a bar to stderr:

```
$ printf '{"current":1,"total":3,"label":"downloading a"}\n\
{"current":2,"total":3,"label":"downloading b"}\n\
{"current":3,"total":3,"label":"done"}\n' \
  | present progress --json
downloading a [########                        ] 33% (1/3)
done: 3/3 (done)
{"done":true}
```

When stdin closes, `present progress` writes `{"done":true}` to stdout and exits `0`.

## What breaks

Measured, not guessed:

- `--ask` without `--options` — exit `1`: `an ask needs options. --options '["a","b"]'`
- Empty options array (`[]`) — exit `1`: `options is empty, present needs at least two to ask`
- Single option (`["only"]`) — exit `1`: `only one option was given, nothing to ask. pass it through or add another`
- Empty message — exit `1`: `message is empty, present needs something to ask`
- Malformed JSON on stdin — exit `1`, names the parse error and the line
- Non-string option elements (`[1,2]`) — exit `1`: `expected a string`
- `--json` mode with no `/dev/tty` and no `PRESENT_AUTO_PICK` — exit `1`, names the missing terminal
  and points at `PRESENT_AUTO_PICK=1` as the next step
- Cancelled pick (empty line, `0`, or `cancel`) — exit `2` in plain mode, `{"choice":null,"cancelled":true}` in json mode
- Out-of-range or non-numeric pick — exit `1`, names the bad input and the valid range
- `--interactive` outside a tty — exit `1`: `present --interactive needs a terminal`
- No args, no tty — exit `1`: `interactive only. pipe a question with --json or run me in a terminal.`
- `sheol` not on PATH (browse/module mode) — exit `1`: `sheol is not on PATH. install it: cargo build --release && install -m755 target/release/sheol ~/.local/bin/sheol`
- Module has no `entry.cli` — exit `1`: `no entry point for <name>. run sheol check <id>`
- `--module <name>` not on shelf — exit `1`: `no module named "<name>" on the shelf. run sheol search to see what is there`
- Progress bar with a bad line — skipped with a stderr warning, the bar keeps running

## The param form limit

`entry.cli` is a string, not a schema. Present tokenizes it (respecting double quotes) and treats
every token after the program as a fillable argument. It cannot tell which token is a script path
and which is a user parameter — it shows all of them with defaults, and you override the ones you
need. If a module's `entry.cli` uses single quotes or shell escapes, the tokenizer will not handle
them correctly in this version.

## Dependencies

`ratatui` and `crossterm` for the interactive picker and shelf browser. `clap` for args. `serde`
for the JSON shape. Nothing touches the network directly — `sheol search --json` is a subprocess.

## What this module is not

- It is not the HTML variant. A browser-facing picker is a separate module (`front`).
- It is not loaded by sheol core. It is a standalone binary on the shelf.
- It is not auto-discovered. A project opts in by calling it explicitly.
