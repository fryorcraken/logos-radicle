# module-settings Specification

## Purpose
The module's own persistent configuration: the five settings that survive a
restart, the per-Basecamp-profile file they live in, and the rule that every
value is validated when it is written rather than when something later reads
it.

## Requirements

### Requirement: The settings store holds exactly five named keys

The module MUST persist exactly five settings, and no other key, each with a
fixed meaning:

- `mode` — which node this module is pointed at: `explore`, `local` or
  `embedded`;
- `radHome` — an explicit Radicle home; `""` means resolve one;
- `radSocket` — an explicit node control socket; `""` means resolve one;
- `gitPath` — an explicit `git` executable; `""` means find one on `PATH`;
- `remoteSeed` — the seed URL the `remote*` methods proxy to; `""` means the
  built-in default.

#### Scenario: Every key is present in a reply even when nothing was written

- **WHEN** `getSettings` is called against a settings file that has never
  been written
- **THEN** the reply MUST contain all five of `mode`, `radHome`, `radSocket`,
  `gitPath` and `remoteSeed`
- **AND** `mode` MUST be `explore`
- **AND** `radHome`, `radSocket`, `gitPath` and `remoteSeed` MUST each be the
  empty string

#### Scenario: A successful write returns the whole settings object

- **WHEN** `setSetting` accepts a value for one key
- **THEN** the reply MUST be the full settings object, containing all five
  keys, not only the key that changed
- **AND** the reply MUST carry no `error` member

#### Scenario: A key the store does not know is refused by name

- **WHEN** `setSetting` is called with the key `noSuchSetting`
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** the error message MUST contain the text `noSuchSetting`
- **AND** no key named `noSuchSetting` MUST appear in a subsequent
  `getSettings` reply

### Requirement: The settings file lives under the XDG data home

The settings file MUST be `<base>/radicle-module/settings.json`, where `<base>`
is `$XDG_DATA_HOME` when that is set and non-empty, and `$HOME/.local/share`
otherwise. When neither is available the resolved path MUST be empty.

The module MUST NOT derive this path from `RAD_HOME` or from any Radicle
profile location, and MUST NOT write its settings into a node-owned file such
as `~/.radicle/config.json`.

#### Scenario: An XDG data home in force decides the path

- **WHEN** the settings path is resolved with an XDG data home of `/xdg/data`
  and a user home of `/home/u`
- **THEN** the path MUST be `/xdg/data/radicle-module/settings.json`

#### Scenario: No XDG data home falls back under the user home

- **WHEN** the settings path is resolved with an empty XDG data home and a
  user home of `/home/u`
- **THEN** the path MUST be
  `/home/u/.local/share/radicle-module/settings.json`

#### Scenario: Two Basecamp profiles resolve two different files

- **WHEN** the settings path is resolved once with an XDG data home of
  `/profiles/alice/xdg-data` and once with `/profiles/bob/xdg-data`
- **THEN** the two paths MUST differ
- **AND** each MUST lie under the XDG data home it was given

#### Scenario: Nothing to resolve from yields no path

- **WHEN** the settings path is resolved with both the XDG data home and the
  user home empty
- **THEN** the resolved path MUST be empty

#### Scenario: Two stores in two places do not see each other's values

- **WHEN** one store at path A persists `remoteSeed` as
  `https://alice.example` and a second store at path B persists it as
  `https://bob.example`
- **THEN** reading `remoteSeed` from the store at A MUST give
  `https://alice.example`
- **AND** reading it from the store at B MUST give `https://bob.example`

#### Scenario: A write through one store leaves an unrelated store at its default

- **WHEN** a store at path A persists `mode` as `local` and a store at path B
  has never had a mode written
- **THEN** reading `mode` from the store at B MUST give `explore`

### Requirement: Settings survive a restart

A value accepted by `setSetting` MUST be readable by a store constructed
afresh over the same path, which is what a module restart amounts to at this
layer. Writing one key MUST NOT disturb the stored value of any other key.

A persisted `remoteSeed` MUST be adopted when the module starts, so that
`getCapabilities().remoteSeed` reports it without any caller re-setting it.

#### Scenario: A value written by one store is read back by a later one

- **WHEN** a store persists `remoteSeed` as `https://persisted.example` and
  is then destroyed
- **AND** a new store is constructed over the same path
- **THEN** the new store MUST report `remoteSeed` as
  `https://persisted.example`

#### Scenario: Writing one key leaves the others untouched

- **WHEN** a store persists `mode` as `local` and then persists `remoteSeed`
  as `https://kept.example`
- **THEN** reading `mode` MUST still give `local`
- **AND** reading `remoteSeed` MUST give `https://kept.example`

#### Scenario: A persisted seed is in force at the next start

- **WHEN** a settings file carries `remoteSeed` of
  `https://persisted.example.test`
- **AND** a module instance is constructed over that file without any further
  seed call
- **THEN** `getCapabilities().remoteSeed` MUST be
  `https://persisted.example.test`

#### Scenario: Adoption reads the settings it was given

- **WHEN** two module instances are constructed over two settings files
  carrying `https://alpha.example.test` and `https://beta.example.test`
  respectively
- **THEN** the first instance's `getCapabilities().remoteSeed` MUST be
  `https://alpha.example.test`
- **AND** the second's MUST be `https://beta.example.test`

### Requirement: Values are validated on write, not on use

`setSetting` MUST validate a value before storing it, and refuse one it cannot
accept with `{"error":"..."}`, leaving the stored value unchanged:

- `mode` MUST be one of `explore`, `local` or `embedded`.
- `gitPath`, when non-empty, MUST be an absolute path to an executable that
  answers as git. An empty value MUST be accepted.
- `radSocket`, when non-empty, MUST be at most 107 bytes.
- `remoteSeed`, when non-empty, MUST begin with `http://` or `https://`.

#### Scenario: A mode the module does not know is refused and lists the valid ones

- **WHEN** `setSetting` is called with key `mode` and value `turbo`
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** the message MUST contain `explore`, `local` and `embedded`
- **AND** a subsequent read of `mode` MUST still give the value that was
  stored before the call

#### Scenario: All three modes are accepted

- **WHEN** `setSetting` is called with key `mode` and each of `explore`,
  `local` and `embedded` in turn
- **THEN** each call MUST return a settings object rather than an error
- **AND** a read of `mode` after each call MUST give the value just written

#### Scenario: A git path that is not a usable git is refused and named

- **WHEN** `setSetting` is called with key `gitPath` and value
  `/definitely/not/here/git`
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** the message MUST contain `/definitely/not/here/git`
- **AND** a subsequent read of `gitPath` MUST give the empty string

#### Scenario: An empty git path is accepted

- **WHEN** `setSetting` is called with key `gitPath` and an empty value
- **THEN** the reply MUST be a settings object rather than an error

#### Scenario: An over-long socket path is refused at write time

- **WHEN** `setSetting` is called with key `radSocket` and a value of 126
  bytes
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** the message MUST state the limit as `107`
- **AND** a subsequent read of `radSocket` MUST give the empty string

#### Scenario: A socket path within the cap is accepted

- **WHEN** `setSetting` is called with key `radSocket` and value
  `/run/user/1000/rad.sock`
- **THEN** the reply MUST be a settings object rather than an error
- **AND** a subsequent read of `radSocket` MUST give
  `/run/user/1000/rad.sock`

#### Scenario: A seed URL with no scheme is refused

- **WHEN** `setSetting` is called with key `remoteSeed` and value
  `seed.radicle.xyz`
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** a subsequent read of `remoteSeed` MUST give the empty string

#### Scenario: Both http and https seed URLs are accepted

- **WHEN** `setSetting` is called with key `remoteSeed` and value
  `https://seed.example`, and again with `http://localhost:8080`
- **THEN** neither reply MUST be an error

#### Scenario: The refusal reaches the caller across the module boundary

- **WHEN** `setSetting` is called through the module's public API with key
  `gitPath` and value `/definitely/not/here/git`
- **THEN** the returned JSON string MUST parse to an object carrying an
  `error` member
- **AND** that member MUST contain `/definitely/not/here/git`

### Requirement: A settings file that cannot be read yields defaults

The module MUST start with working defaults rather than failing when the
settings file is absent, unreadable, or does not parse as a JSON object. A
store constructed with an empty path MUST likewise report defaults, and MUST
refuse a write, with an error naming both `XDG_DATA_HOME` and `HOME`, rather
than report success.

Keys present in the file that the store does not know MUST NOT change any
setting the store reports, and MUST NOT appear in what `getSettings` reports.

#### Scenario: A missing file yields defaults

- **WHEN** a store is constructed over a path where no file exists
- **THEN** reading `mode` MUST give `explore`

#### Scenario: A file that is not JSON yields defaults rather than a dead module

- **WHEN** the settings file contains `{ this is not json at all`
- **THEN** constructing a store over it MUST succeed
- **AND** reading `mode` MUST give `explore`

#### Scenario: An unknown key in the file is ignored

- **WHEN** the settings file contains
  `{"mode":"explore","somethingElse":"x"}`
- **THEN** reading `mode` MUST give `explore`
- **AND** the reported settings MUST NOT contain a key `somethingElse`

#### Scenario: A store with no path reports defaults and refuses to write

- **WHEN** a store is constructed with an empty path
- **THEN** reading `mode` MUST give `explore`
- **AND** a write of `mode` with value `local` MUST return
  `{"error":"..."}`
- **AND** that error message MUST contain `XDG_DATA_HOME`
- **AND** that error message, with every occurrence of `XDG_DATA_HOME`
  removed, MUST still contain `HOME`
- **AND** a subsequent read of `mode` MUST still give `explore`

### Requirement: A stored mode this build cannot interpret reads back as explore

A stored `mode` this build cannot interpret MUST be reported as `explore`,
rather than passed through to a caller or reported as `local`.

Sanitising it MUST NOT discard the file's other settings, and MUST NOT rewrite
the file: the stored string MUST stay as it was found. A mode the build does
know MUST be reported unchanged.

#### Scenario: An uninterpretable stored mode is reported as explore

- **WHEN** the settings file contains
  `{"mode":"turbo","remoteSeed":"https://kept.example.test"}`
- **THEN** reading `mode` MUST give `explore`
- **AND** reading `remoteSeed` MUST give `https://kept.example.test`

#### Scenario: Each known mode is reported unchanged

- **WHEN** the settings file contains `{"mode":"local"}`, then
  `{"mode":"embedded"}`, then `{"mode":"explore"}` in turn
- **THEN** reading `mode` after each MUST give exactly the mode that was in
  the file

#### Scenario: The unreadable-mode fallback is not a file rewrite

- **WHEN** a store reads a settings file containing `{"mode":"turbo"}`
- **AND** no write is performed through that store
- **THEN** the file on disk MUST still contain the mode string `turbo`

### Requirement: A settings write that repoints the module is applied at once

Writing `mode`, `radHome` or `radSocket` MUST take effect on the running
instance: afterwards `getCapabilities` MUST report the home and availability of
the newly selected mode, not of the previous one. Writing `remoteSeed` MUST
adopt the URL for subsequent `remote*` calls.

Writing `gitPath` MUST NOT change the `git` in force on the running instance. It
MUST be applied at the next module start, and a caller MUST be told so.

#### Scenario: Switching mode changes the reported home in the same session

- **WHEN** `setSetting` writes `mode` as `local` against a resolvable Radicle
  home, and `getCapabilities` is read
- **AND** `setSetting` then writes `mode` as `embedded`, and
  `getCapabilities` is read again
- **THEN** the two replies MUST report different `radHome` values
- **AND** the second reply's `mode` MUST be `embedded`

#### Scenario: Switching to explore reports no home at all

- **WHEN** `setSetting` writes `mode` as `local` against a resolvable Radicle
  home, and then writes `mode` as `explore`
- **THEN** `getCapabilities().radHome` MUST be empty
- **AND** `getCapabilities().localAvailable` MUST be false

#### Scenario: The settings panel states the git path's restart-to-apply terms

- **WHEN** the settings panel is shown
- **THEN** it MUST display text saying that a changed git path is checked
  immediately but takes effect the next time the module is started

### Requirement: A view renders settings from the reply it was given

A view MUST populate itself from a `getSettings` reply rather than from its
own assumptions, and MUST re-render from the object a successful `setSetting`
returns rather than from the value it submitted.

A refusal MUST be surfaced to the user verbatim — the messages name the path
that was tried and the limit that was exceeded, which is the whole reason
validation happens on write — and the displayed settings MUST NOT change. A
subsequent successful write MUST clear a previously displayed refusal.

#### Scenario: The panel shows the persisted settings

- **WHEN** the settings panel is loaded against a store holding `mode` of
  `local`
- **THEN** the panel MUST report its current mode as `local`

#### Scenario: Choosing a mode persists it and the panel follows

- **WHEN** the user chooses a mode in the panel
- **THEN** the panel MUST issue a `setSetting` write for `mode` with that
  value
- **AND** the panel's reported current mode MUST become the mode written

#### Scenario: Two different mode choices give two different panel states

- **WHEN** the user chooses `embedded`, and then chooses `explore`
- **THEN** the panel's reported current mode MUST be `embedded` after the
  first choice and `explore` after the second

#### Scenario: A refused git path is shown and changes nothing

- **WHEN** the user saves a git path the backend refuses
- **THEN** the panel MUST display the refusal message returned by the backend
- **AND** the panel's reported current git path MUST be unchanged

#### Scenario: A refused mode shows its own message

- **WHEN** a mode write is refused with a message naming the mode
- **THEN** the panel MUST display that message rather than the git path's
  message

#### Scenario: A successful write clears a previous refusal

- **WHEN** the panel is displaying a refusal message
- **AND** a subsequent write is accepted
- **THEN** the panel MUST no longer report an error

### Requirement: A configured git path is proved to be git, and never falls back

A non-empty `gitPath` MUST be accepted only when the path it names both runs
and identifies itself as git. Each of these MUST be refused, with a refusal
naming the value tried and what was wrong with it: a path that does not exist;
an executable that exits zero but whose `--version` output does not identify
it as git; one whose `--version` exits non-zero; a path that is not absolute.

A configured path that fails MUST NOT fall back to a git found on `PATH`.

#### Scenario: A real-but-not-git executable is refused

- **WHEN** `gitPath` is set to an absolute path to an executable that exits
  zero and prints something other than a git version
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** the message MUST say that the binary does not look like git
- **AND** a subsequent read of `gitPath` MUST give the value stored before the
  call

#### Scenario: An executable that exits non-zero is refused

- **WHEN** `gitPath` is set to an absolute path to an executable that exits
  non-zero when asked for its version
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** the message MUST name that path

#### Scenario: A relative path is refused even when it names a working git

- **WHEN** `gitPath` is set to a relative path
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** the message MUST say the path must be absolute
- **AND** the refusal MUST hold for a relative path that names a real, working
  git, so that refusing relative paths is distinguishable from refusing
  everything

#### Scenario: The same binary is accepted by its absolute path

- **WHEN** `gitPath` is set to a relative path naming a real, working git, and
  then to that same binary's absolute path
- **THEN** the first call MUST be refused
- **AND** the second MUST NOT be an error
- **AND** the reported resolved path MUST be that absolute path

#### Scenario: A refused configured path does not silently use PATH

- **WHEN** a working git exists on `PATH`
- **AND** `gitPath` is set to an absolute path that does not exist
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** the message MUST name the path that was tried rather than the one on
  `PATH`
- **AND** `getCapabilities().gitFound` MUST NOT be reported as true on the
  strength of the `PATH` git for that configured value

### Requirement: The reported git path distinguishes configured from detected

`getCapabilities()` MUST report `gitConfigured` as true exactly when a non-empty
`gitPath` setting is in force, including one that was refused, and false when
git was found on `PATH`.

`gitPath` MUST report the resolved absolute path when git was found, and the
empty string when it was not. `gitProblem` MUST be empty exactly when `gitFound`
is true.

#### Scenario: A detected git is not reported as configured

- **WHEN** the `gitPath` setting is empty and a working git is found on `PATH`
- **THEN** `getCapabilities().gitFound` MUST be true
- **AND** `gitConfigured` MUST be false
- **AND** `gitPath` MUST be a non-empty absolute path
- **AND** `gitProblem` MUST be empty

#### Scenario: A refused configured path is still reported as configured

- **WHEN** a non-empty `gitPath` is in force that does not resolve to a usable
  git
- **THEN** `getCapabilities().gitFound` MUST be false
- **AND** `gitConfigured` MUST be true
- **AND** `gitProblem` MUST be non-empty

### Requirement: getSettings reports the five keys and no other

`getSettings` MUST return all five keys on every call, with defaults filled in
for anything unset, and MUST NOT return any other key.

#### Scenario: A reply carries exactly five keys

- **WHEN** `getSettings` is called against a settings file that has never been
  written, and again after `setSetting` has accepted a value for each key
- **THEN** each reply MUST contain exactly five keys

### Requirement: A settings refusal names what was wrong

A refused `mode` MUST name all three modes, a refused `gitPath` MUST name the
value that was tried, and a refused `radSocket` MUST name the 107-byte limit.

#### Scenario: A refused mode names all three whatever was tried

- **WHEN** `setSetting` is called with key `mode` and value `remote`
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** the message MUST contain `explore`, `local` and `embedded`

### Requirement: A seed URL is checked for its shape only

`setSetting` MUST check a `remoteSeed` value for its shape only: it MUST NOT
make a network round trip to learn whether the seed answers. Proving that a
seed answers is `setRemoteSeed`'s, which stores a seed only once proved, under
"A seed setRemoteSeed refuses changes nothing".

#### Scenario: A seed that cannot be reached is accepted

- **WHEN** `setSetting` is called with key `remoteSeed` and value
  `https://seed.invalid`
- **THEN** the reply MUST be a settings object rather than an error
- **AND** a subsequent read of `remoteSeed` MUST give `https://seed.invalid`

### Requirement: A seed accepted by setRemoteSeed survives a restart

When `setRemoteSeed` accepts a seed, it MUST store the normalised URL, as
"An accepted seed is reported as its normalised URL" defines it, as the
`remoteSeed` setting before it replies. Afterwards `getSettings().remoteSeed`
MUST give that URL, and a module instance constructed afresh over the same
settings file MUST have it in force, by the adoption "Settings survive a
restart" requires.

#### Scenario: A chosen seed is in force after a restart

- **WHEN** a module instance is constructed over a settings file that has
  never been written
- **AND** `setRemoteSeed` is called with `https://chosen.example.test` and that
  seed answers its probe
- **AND** a new module instance is constructed over the same settings file,
  with no further seed call
- **THEN** the new instance's `getCapabilities().remoteSeed` MUST be
  `https://chosen.example.test`

#### Scenario: A chosen seed is stored before the call returns

- **WHEN** `setRemoteSeed` is called with `https://chosen.example.test` and that
  seed answers its probe
- **THEN** the reply MUST carry no `error` member
- **AND** a `getSettings` call on the same instance MUST report `remoteSeed` as
  `https://chosen.example.test`

#### Scenario: The later of two accepted seeds is the one stored

- **WHEN** `setRemoteSeed` is called with `https://first.example.test` and then
  with `https://second.example.test`, and both answer their probes
- **AND** a new module instance is constructed over the same settings file
- **THEN** the new instance's `getCapabilities().remoteSeed` MUST be
  `https://second.example.test`

### Requirement: An accepted seed is reported as its normalised URL

The normalised URL is the URL `setRemoteSeed` was given with every trailing `/`
removed. On accepting a seed, `setRemoteSeed` MUST reply with an object whose
`seed` is the normalised URL, whose `apiVersion` is the probe's `apiVersion`,
and whose `nid` is the probe's `nid` when that is a string and empty otherwise.
`getCapabilities().remoteSeed` MUST then be the normalised URL, and the call
MUST emit exactly one `remoteSeedChanged`, carrying it.

#### Scenario: A trailing slash is removed wherever the seed is reported

- **WHEN** `setRemoteSeed` is called with `https://chosen.example.test/` and
  that seed answers its probe
- **THEN** the reply's `seed` MUST be `https://chosen.example.test`
- **AND** `getCapabilities().remoteSeed` MUST be `https://chosen.example.test`
- **AND** `getSettings().remoteSeed` MUST be `https://chosen.example.test`

#### Scenario: An accepted seed emits one event naming it

- **WHEN** `setRemoteSeed` is called with `https://chosen.example.test` and
  that seed answers its probe
- **THEN** exactly one `remoteSeedChanged` event MUST have been emitted by the
  call
- **AND** its payload MUST be `https://chosen.example.test`

#### Scenario: The reply carries what the probe learned

- **WHEN** `setRemoteSeed` is called with `https://chosen.example.test` and the
  probe's reply body is `{"apiVersion":"6.2.0","nid":"z6MkChosen"}`
- **THEN** the reply's `apiVersion` MUST be `6.2.0`
- **AND** the reply's `nid` MUST be `z6MkChosen`

#### Scenario: A nid that is absent or not a string is reported empty

- **WHEN** `setRemoteSeed` is called with `https://chosen.example.test` and the
  probe's reply body is `{"apiVersion":"6.2.0"}`, and again with
  `{"apiVersion":"6.2.0","nid":5}`
- **THEN** neither reply MUST carry an `error` member
- **AND** each reply's `nid` MUST be the empty string

### Requirement: A seed answers its probe only with a Radicle seed index

`setRemoteSeed`'s probe is one GET of `<normalised URL>/api/v1`. A seed MUST be
taken to answer only on a 2xx status with a body that is a JSON object holding
a string `apiVersion` and no `error` member. Anything else is a failed probe: a
transport error, another status, a body that is not a JSON object, or an object
failing either test. A failed probe MUST be refused as "A seed setRemoteSeed
refuses changes nothing" requires, replying with only an `error` member, a
non-empty string.

#### Scenario: A body carrying a string apiVersion answers

- **WHEN** `setRemoteSeed` is called with `https://chosen.example.test` and the
  probe succeeds with the body `{"apiVersion":"6.2.0"}`
- **THEN** the reply MUST carry no `error` member
- **AND** `getSettings().remoteSeed` MUST be `https://chosen.example.test`

#### Scenario: A success that is not a seed index is a failed probe

- **WHEN** a settings file carries `remoteSeed` of `https://kept.example.test`
  and a module instance is constructed over it
- **AND** `setRemoteSeed` is called with `https://other.example.test`, whose
  probe succeeds with a 2xx status and, in turn, each of the bodies `[]`,
  `null`, `"ok"`, `42`, `{}`, `{"apiVersion":6}` and `not json`
- **THEN** each reply MUST be `{"error":"..."}`
- **AND** after each, `getCapabilities().remoteSeed` and
  `getSettings().remoteSeed` MUST be `https://kept.example.test`
- **AND** no `remoteSeedChanged` event MUST have been emitted by any of the
  calls

#### Scenario: A success carrying an error member is a failed probe

- **WHEN** a settings file carries `remoteSeed` of `https://kept.example.test`
  and a module instance is constructed over it
- **AND** `setRemoteSeed` is called with `https://other.example.test`, whose
  probe succeeds with a 2xx status and, in turn, each of the bodies
  `{"apiVersion":"6.2.0","error":"x"}`, `{"apiVersion":"6.2.0","error":null}`,
  `{"apiVersion":"6.2.0","error":""}`, `{"error":false}` and `{"error":5}`
- **THEN** each reply's `error` MUST be a non-empty string
- **AND** each reply MUST carry no member other than `error`
- **AND** after each, `getCapabilities().remoteSeed` and
  `getSettings().remoteSeed` MUST be `https://kept.example.test`
- **AND** no `remoteSeedChanged` event MUST have been emitted by any of the
  calls

#### Scenario: A failure status is a failed probe whatever its body

- **WHEN** a settings file carries `remoteSeed` of `https://kept.example.test`
  and a module instance is constructed over it
- **AND** `setRemoteSeed` is called with `https://other.example.test`, whose
  probe fails with status 500 and, in turn, each of the bodies
  `{"apiVersion":"6.2.0"}`, `{"error":5}` and `{"error":""}`
- **THEN** each reply's `error` MUST be a non-empty string
- **AND** each reply MUST carry no member other than `error`
- **AND** after each, `getSettings().remoteSeed` MUST be
  `https://kept.example.test`

### Requirement: A seed setRemoteSeed refuses changes nothing

`setRemoteSeed` MUST refuse with `{"error":"..."}` an empty URL, a URL whose
given or normalised form `setSetting` would refuse for `remoteSeed`, a seed
that fails its probe, and a seed it cannot store. A refusal MUST leave the seed in force as it was,
including the `remoteReachable` and `remoteApiVersion` that `getCapabilities`
reports for it, MUST leave the stored `remoteSeed` as it was, and MUST NOT emit
`remoteSeedChanged`.

#### Scenario: A failed probe stores nothing

- **WHEN** a settings file carries `remoteSeed` of `https://kept.example.test`
  and a module instance is constructed over it
- **AND** `setRemoteSeed` is called with `https://unreachable.example.test` and
  the probe fails
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** `getCapabilities().remoteSeed` MUST be `https://kept.example.test`
- **AND** `getSettings().remoteSeed` MUST be `https://kept.example.test`
- **AND** a new module instance constructed over the same settings file MUST
  report `getCapabilities().remoteSeed` as `https://kept.example.test`
- **AND** no `remoteSeedChanged` event MUST have been emitted by the call

#### Scenario: A failed probe leaves the seed in force reachable

- **WHEN** the seed in force is `https://good.example.test`, it has answered a
  probe with `apiVersion` `6.2.0`, and `getCapabilities().remoteReachable` is
  true
- **AND** `setRemoteSeed` is called with `https://unreachable.example.test` and
  the probe fails
- **THEN** `getCapabilities().remoteSeed` MUST be `https://good.example.test`
- **AND** `getCapabilities().remoteReachable` MUST be true
- **AND** `getCapabilities().remoteApiVersion` MUST be `6.2.0`

#### Scenario: A seed that answers but cannot be stored is refused

- **WHEN** a module instance's settings store refuses every write, as a store
  constructed with no path does, and the seed in force is
  `https://good.example.test`
- **AND** `setRemoteSeed` is called with `https://answers.example.test` and that
  seed answers its probe
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** `getCapabilities().remoteSeed` MUST be `https://good.example.test`
- **AND** no `remoteSeedChanged` event MUST have been emitted by the call

#### Scenario: A URL with no scheme is refused even when the seed would answer

- **WHEN** a settings file carries `remoteSeed` of `https://kept.example.test`
  and a module instance is constructed over it
- **AND** every request to any seed would succeed
- **AND** `setRemoteSeed` is called with `seed.example.test`
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** `getCapabilities().remoteSeed` MUST be `https://kept.example.test`
- **AND** `getSettings().remoteSeed` MUST be `https://kept.example.test`

#### Scenario: An empty URL is refused and leaves the stored seed alone

- **WHEN** a settings file carries `remoteSeed` of `https://kept.example.test`
  and a module instance is constructed over it
- **AND** `setRemoteSeed` is called with an empty URL
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** `getCapabilities().remoteSeed` MUST be `https://kept.example.test`
- **AND** `getSettings().remoteSeed` MUST be `https://kept.example.test`

### Requirement: A malformed seed URL is refused before any request

`setRemoteSeed` MUST refuse, without making any request, a URL for which
`setSetting` would refuse either the URL as given or its normalised URL for
`remoteSeed`, and the refusal MUST carry the reason `setSetting` gives. Such a
URL MUST be refused whether or not the seed it names would answer a probe.

#### Scenario: A URL with no scheme costs no request and says why

- **WHEN** every request to any seed would succeed
- **AND** `setRemoteSeed` is called with `seed.example.test`
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** its message MUST contain `http://` and `https://`
- **AND** no request MUST have been made

#### Scenario: A scheme with no host is refused before any request

- **WHEN** a settings file carries `remoteSeed` of `https://kept.example.test`
  and a module instance is constructed over it
- **AND** every request to any seed would succeed
- **AND** `setRemoteSeed` is called with `https://`, and again with `http://`
- **THEN** each reply MUST be `{"error":"..."}`
- **AND** each message MUST contain `http://` and `https://`
- **AND** no request MUST have been made
- **AND** `getSettings().remoteSeed` MUST be `https://kept.example.test`

### Requirement: A seed URL carrying credentials is refused

A URL's authority is the text between its `://` and the first `/`, `?` or `#`
after it, and its userinfo is the authority's text up to its last `@`.
`setSetting` MUST refuse a non-empty `remoteSeed` whose authority contains `@`,
and the refusal MUST NOT contain the userinfo. `setRemoteSeed` refuses such a
URL before any request, by "A malformed seed URL is refused before any
request".

#### Scenario: A seed URL with credentials is refused and its secret not echoed

- **WHEN** `setSetting` is called with key `remoteSeed` and value
  `https://user:s3cr3t@seed.example.test`, and again with
  `https://s3cr3t@seed.example.test`
- **THEN** each reply MUST be `{"error":"..."}`
- **AND** neither message MUST contain `s3cr3t`
- **AND** a subsequent read of `remoteSeed` MUST give the value stored before
  the calls

#### Scenario: An @ after the host is not a credential

- **WHEN** `setSetting` is called with key `remoteSeed` and value
  `https://seed.example.test/a@b`
- **THEN** the reply MUST be a settings object rather than an error

#### Scenario: setRemoteSeed sends no request for a URL with credentials

- **WHEN** a settings file carries `remoteSeed` of `https://kept.example.test`
  and a module instance is constructed over it
- **AND** every request to any seed would succeed
- **AND** `setRemoteSeed` is called with `https://user:s3cr3t@seed.example.test`
- **THEN** the reply MUST be `{"error":"..."}`
- **AND** its message MUST NOT contain `s3cr3t`
- **AND** no request MUST have been made
- **AND** `getSettings().remoteSeed` and `getCapabilities().remoteSeed` MUST be
  `https://kept.example.test`

### Requirement: A settings write changes only the key it names

An accepted settings write, by `setSetting` or by `setRemoteSeed` storing a
seed, MUST leave every other member of the settings file with the JSON value
it had, unknown, uninterpretable and non-string ones included, unless "A
settings write drops a member nested too deep to keep" drops it. It MUST add no
other member. A refused write MUST leave the file's bytes unchanged. A kept
member is not one the module persists, as "The settings store holds exactly
five named keys" means it.

#### Scenario: A write of one key keeps an unknown mode and an unknown key

- **WHEN** the settings file contains
  `{"mode":"turbo","remoteSeed":"https://a.example","futureKey":"x"}`
- **AND** `setSetting` is called with key `radHome` and value
  `/home/u/.radicle`, and accepts it
- **THEN** the file on disk MUST parse as a JSON object equal to
  `{"mode":"turbo","remoteSeed":"https://a.example","futureKey":"x","radHome":"/home/u/.radicle"}`

#### Scenario: A write keeps a known key whose value is not a string

- **WHEN** the settings file contains
  `{"mode":"local","radSocket":5,"gitPath":null}`
- **AND** `setSetting` is called with key `remoteSeed` and value
  `https://b.example`, and accepts it
- **THEN** the file on disk MUST parse as a JSON object equal to
  `{"mode":"local","radSocket":5,"gitPath":null,"remoteSeed":"https://b.example"}`

#### Scenario: Writing the key that held an uninterpretable value replaces only that value

- **WHEN** the settings file contains `{"mode":"turbo","futureKey":"x"}`
- **AND** `setSetting` is called with key `mode` and value `local`, and
  accepts it
- **THEN** the file on disk MUST parse as a JSON object equal to
  `{"mode":"local","futureKey":"x"}`

#### Scenario: A refused write leaves the file as it was

- **WHEN** the settings file contains `{"mode":"turbo","futureKey":"x"}`
- **AND** `setSetting` is called with key `mode` and value `remote`, and
  again with key `noSuchSetting` and value `y`
- **THEN** each reply MUST be `{"error":"..."}`
- **AND** after each, the file's bytes MUST equal the bytes it held before
  the first call
- **AND** after each, the file on disk MUST parse as a JSON object equal to
  `{"mode":"turbo","futureKey":"x"}`

#### Scenario: A seed stored by setRemoteSeed keeps the file's other members

- **WHEN** the settings file contains
  `{"mode":"turbo","remoteSeed":"https://kept.example.test","futureKey":"x"}`
  and a module instance is constructed over it
- **AND** `setRemoteSeed` is called with `https://chosen.example.test` and that
  seed answers its probe
- **THEN** the file on disk MUST parse as a JSON object equal to
  `{"mode":"turbo","remoteSeed":"https://chosen.example.test","futureKey":"x"}`

### Requirement: A settings write keeps each member's value, not the file's text

An accepted write MUST keep what "A settings write changes only the key it
names" requires whatever the file's member order, whitespace or number
spelling. A member the file names more than once, other than the one written,
MUST be kept as one member holding the last value the file gives it. The
file's text, meaning its member order, whitespace and number spelling, is
outside that requirement, and so is the value of a number beyond the range or
precision of an IEEE 754 binary64 double.

#### Scenario: The file's layout does not change what a write keeps

- **WHEN** the settings file contains `{"futureKey":1.5e3,"mode":"turbo"}`,
  written with a newline after `{` and after each `,`, a tab before each
  member, and a space on each side of each `:`
- **AND** `setSetting` is called with key `radHome` and value
  `/home/u/.radicle`, and accepts it
- **THEN** the file on disk MUST parse as a JSON object equal to
  `{"futureKey":1.5e3,"mode":"turbo","radHome":"/home/u/.radicle"}`

#### Scenario: A member the file names twice is kept once, with its last value

- **WHEN** the settings file contains
  `{"futureKey":"a","mode":"turbo","futureKey":"b"}`
- **AND** `setSetting` is called with key `radHome` and value
  `/home/u/.radicle`, and accepts it
- **THEN** the file on disk MUST parse as a JSON object equal to
  `{"mode":"turbo","futureKey":"b","radHome":"/home/u/.radicle"}`
- **AND** the file's text MUST contain `"futureKey"` exactly once

### Requirement: A settings write drops a member nested too deep to keep

A value's depth is the number of arrays and objects on its deepest path,
counting the value itself when it is one: `"x"` has depth 0, `[]` depth 1 and
`{"a":[]}` depth 2. Of the members other than the one written, an accepted
write MUST keep every one of depth 128 or less and MUST drop from the file
every one deeper. However deep a member nests, the write MUST complete and
reply as any accepted write does, and MUST NOT be refused for that depth alone.

#### Scenario: A member exactly 128 deep is kept

- **WHEN** the settings file contains `{"mode":"turbo","deep":D}`, where `D` is
  `[` written 128 times followed by `]` written 128 times
- **AND** `setSetting` is called with key `radHome` and value
  `/home/u/.radicle`, and accepts it
- **THEN** the file on disk MUST parse as a JSON object equal to
  `{"mode":"turbo","deep":D,"radHome":"/home/u/.radicle"}`

#### Scenario: A member 129 deep is dropped and the rest kept

- **WHEN** the settings file contains `{"mode":"turbo","deep":D}`, where `D` is
  `[` written 129 times followed by `]` written 129 times
- **AND** `setSetting` is called with key `radHome` and value
  `/home/u/.radicle`, and accepts it
- **THEN** the file on disk MUST parse as a JSON object equal to
  `{"mode":"turbo","radHome":"/home/u/.radicle"}`

#### Scenario: A member nested far deeper still lets the write complete

- **WHEN** the settings file contains `{"mode":"turbo","deep":D}`, where `D` is
  `[` written 100000 times followed by `]` written 100000 times
- **AND** `setSetting` is called with key `radHome` and value
  `/home/u/.radicle`
- **THEN** the reply MUST be a settings object rather than an error, with
  `radHome` `/home/u/.radicle`
- **AND** the file on disk MUST parse as a JSON object equal to
  `{"mode":"turbo","radHome":"/home/u/.radicle"}`

### Requirement: A settings write replies with the settings as read, not the file as stored

The reply to an accepted `setSetting` MUST equal what `getSettings` reports
immediately afterwards. It MUST NOT carry a member of the file that this build
does not know, and it MUST report a stored `mode` this build cannot interpret
as `explore`, even though the file keeps that mode as "A settings write
changes only the key it names" requires.

#### Scenario: The reply is the normalised view while the file keeps its members

- **WHEN** the settings file contains
  `{"mode":"turbo","remoteSeed":"https://a.example","futureKey":"x"}`
- **AND** `setSetting` is called with key `radHome` and value
  `/home/u/.radicle`, and accepts it
- **THEN** the reply MUST contain exactly the five keys `mode`, `radHome`,
  `radSocket`, `gitPath` and `remoteSeed`
- **AND** the reply's `mode` MUST be `explore`
- **AND** the reply's `remoteSeed` MUST be `https://a.example`
- **AND** the reply's `radHome` MUST be `/home/u/.radicle`
- **AND** the reply MUST equal the reply of a `getSettings` call made next
- **AND** the file on disk MUST still hold `mode` as `turbo` and `futureKey` as
  `x`

### Requirement: A settings write over an unreadable file stores only the key written

When the settings file is absent, cannot be read, or does not parse as a JSON
object, an accepted write MUST replace it with a JSON object holding exactly
one member: the key written, with the value written. The state of such a file
MUST NOT by itself cause the write to be refused.

#### Scenario: A write over a file that is not JSON replaces it

- **WHEN** the settings file contains `{ this is not json at all`
- **AND** `setSetting` is called with key `mode` and value `local`
- **THEN** the reply MUST be a settings object rather than an error
- **AND** the file on disk MUST parse as a JSON object equal to
  `{"mode":"local"}`

#### Scenario: A write over JSON that is not an object replaces it

- **WHEN** the settings file contains `["mode","turbo"]`
- **AND** `setSetting` is called with key `mode` and value `local`
- **THEN** the reply MUST be a settings object rather than an error
- **AND** the file on disk MUST parse as a JSON object equal to
  `{"mode":"local"}`

#### Scenario: The first write creates a file holding only that key

- **WHEN** no settings file exists at the store's path
- **AND** `setSetting` is called with key `remoteSeed` and value
  `https://a.example`
- **THEN** the file on disk MUST parse as a JSON object equal to
  `{"remoteSeed":"https://a.example"}`
- **AND** reading `mode` MUST give `explore`
