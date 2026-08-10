# present

> Surfaces a decision to the human and returns the pick, so an agent or module never prints
> "please choose" to stdout and parses free text back.

When an agent hits a fork — pick a module, pick a thumbnail style, pick a provider — it used to
print "which one?" to stdout and try to read the human's reply out of its own conversation log.
That is fragile. `present` does that one job: it takes a message and a list, asks the human through
a real terminal, and returns the pick as a clean value on stdout.

It is opt-in. It is never loaded by default. A sheol project declares `present.ask: true` in its
`sheol.json` when it wants setup decisions surfaced this way; without that field, no project pulls
present in. The lesson is the one opencode paid for: a TUI always loaded is a slow CLI.

## Input

Two shapes, your choice:

- Flags — `present ask --message "<words>" --options '["a","b","c"]' [--multiple] [--interactive]`
- JSON on stdin — `present ask --json`, reads `{"message":"<words>","options":["a","b","c"],"multiple":false}`

In `--json` mode the request comes from stdin and the human's pick comes from `/dev/tty` — that
is how an agent pipes the request while the human still types the answer.

## Output

- Plain mode — the picked option on stdout, exit `0`. Cancelled: empty stdout, exit `2`. Bad input: exit `1`.
- JSON mode — one line on stdout:
  - picked: `{"selected":["a"]}` (always an array; one entry when not `--multiple`)
  - cancelled: `{"cancelled":true}`
  - bad input: `{"error":"...","code":"bad-input"}`, exit `1`

```
$ printf '{"message":"pick a thumbnail","options":["bold","minimal","photo"]}' \
  | PRESENT_AUTO_PICK=1 present ask --json
{"selected":["bold"]}
```

`PRESENT_AUTO_PICK=1` makes `--json` mode auto-select the first option without reading `/dev/tty`.
It exists for non-interactive environments (CI, agents with no attached terminal). Don't set it
when a human is present — they would never see the prompt.

## The exact line

```
present ask --message "Pick a thumbnail" --options '["bold","minimal","photo"]'
```

## Multi-select

```
$ present ask --message "Pick modules to install" --options '["a","b","c"]' --multiple
pick one or more by number, comma-separated. 0 or empty cancels
1,3
a,c
```

In `--json` mode the response is `{"selected":["a","c"]}`.

## Progress strip

A second subcommand reads `{current,total,label}` JSON lines from stdin and renders a bar to
stderr. It is for background-install style flows where one process feeds updates and the human
watches the bar move.

```
$ printf '{"current":1,"total":3,"label":"downloading a"}\n\
{"current":2,"total":3,"label":"downloading b"}\n\
{"current":3,"total":3,"label":"done"}\n' \
  | present progress --json
downloading a [########                        ] 33% (1/3)
done: 3/3 (done)
{"done":true}
```

When stdin closes, `present progress` writes `{"done":true}` to stdout and exits `0`. A line that
is not valid JSON is skipped with a warning on stderr, not fatal.

## Interactive picker

`--interactive` forces a ratatui list: arrows to move, enter to confirm, esc to cancel, space to
toggle when `--multiple`. If `--interactive` is passed outside a terminal, present exits `1` with
a message — it does not silently fall back, because silent fallback is the magic the project
refused to ship.

## What breaks

Measured, not guessed:

- Empty options array (`[]`) — exit `1`, stdout (json) or stderr (cli): `options is empty, present needs at least two to ask`.
- Single option (`["only"]`) — exit `1`, `only one option was given, nothing to ask. pass it through or add another`.
- Empty message — exit `1`, `message is empty, present needs something to ask`.
- Malformed JSON on stdin (`{not json`) — exit `1`, names the parse error and the line.
- Options array with non-string elements (`[1,2]`) — exit `1`, `invalid type: integer \`1\`, expected a string`.
- `--json` mode with no `/dev/tty` and no `PRESENT_AUTO_PICK` — exit `1`, names the missing
  terminal and points at `PRESENT_AUTO_PICK=1` as the next step. This is the failure mode for an
  agent running in a sandbox with no attached human.
- Cancelled pick (empty line, `0`, or `cancel`) — exit `2` in plain mode, `{"cancelled":true}` in
  json mode. Cancel is not an error.
- Out-of-range or non-numeric pick — exit `1`, names the bad input and the valid range.
- `--interactive` outside a tty — exit `1`, `present --interactive needs a terminal. drop the flag
  to use the plain prompt, or run it in a tty`.
- Progress bar with a line that fails to parse — skipped with a stderr warning, the bar keeps
  running. One bad update does not kill the strip.
- 100+ options — works in plain and json modes (the list is just longer on stderr). The
  interactive picker does not paginate in this version; a very long list scrolls off the screen.

## How an agent uses it

The contract is the JSON shape. An agent that hits a fork in a script pipes the request and reads
the response:

```
choice=$(printf '%s' "$request_json" | present ask --json)
```

The agent never parses prose. The agent never prints "please choose" to its own stdout. The human
picks in a real terminal, present returns the value, the agent continues.

## The sheol.json opt-in

A project that wants its setup decisions surfaced through present adds one field to its own
`sheol.json`:

```json
{ "present": { "ask": true } }
```

That field is read by sheol core at `sheol pull` time, not by present itself. present is the
surface; sheol core decides when to call it. Without that field on the project, present is never
invoked — it stays on the shelf until a project asks for it.

## Dependencies

`ratatui` and `crossterm` for the `--interactive` picker. `clap` for args. `serde` for the JSON
shape. Nothing touches the network.

## What this module is not

- It is not the HTML variant. A browser-facing picker is a separate module (`front`), different
  concern, different surface.
- It is not auto-discovered. The opt-in field is on the project, not implicit in the tool.
- It is not loaded by sheol core. sheol core reads `present.ask` and calls present when the field
  is set; without it, the binary sits on the shelf unused.
