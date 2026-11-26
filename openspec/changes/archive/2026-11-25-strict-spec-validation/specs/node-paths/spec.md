## MODIFIED Requirements

### Requirement: Socket precedence prefers the runtime directory over the home

`resolveSocket` MUST return the configured socket when it is non-empty;
otherwise the `RAD_SOCKET` value when that is non-empty; otherwise
`<runtime dir>/radicle-<profile>.sock` when a runtime directory is given;
otherwise `<home>/node/control.sock`, a fallback that MUST be retained. When the
profile name is empty the runtime-directory form MUST be
`<runtime dir>/radicle.sock`. When neither a runtime directory nor a home is
available the result MUST be the empty string.

#### Scenario: An explicit socket wins over every other source

- **WHEN** `resolveSocket` is given the configured socket `/chosen.sock`,
  `RAD_SOCKET` of `/env.sock`, a runtime directory `/run/user/1000`, the
  profile `alice` and the home `/home/u/.radicle`
- **THEN** the result MUST be `/chosen.sock`

#### Scenario: RAD_SOCKET wins over the runtime directory

- **WHEN** `resolveSocket` is given no configured socket, `RAD_SOCKET` of
  `/env.sock`, a runtime directory `/run/user/1000`, the profile `alice` and
  the home `/home/u/.radicle`
- **THEN** the result MUST be `/env.sock`
- **AND** it MUST NOT be `/run/user/1000/radicle-alice.sock`

#### Scenario: The runtime directory is preferred over the home

- **WHEN** `resolveSocket` is given no configured socket, no `RAD_SOCKET`, a
  runtime directory `/run/user/1000`, the profile `alice` and the home
  `/home/u/.radicle`
- **THEN** the result MUST be `/run/user/1000/radicle-alice.sock`
- **AND** it MUST NOT be `/home/u/.radicle/node/control.sock`

#### Scenario: Two profiles get two sockets in one runtime directory

- **WHEN** `resolveSocket` is called twice with the same runtime directory
  `/run/user/1000` and the same home, once with the profile `alice` and once
  with `bob`
- **THEN** the two results MUST be `/run/user/1000/radicle-alice.sock` and
  `/run/user/1000/radicle-bob.sock` respectively
- **AND** they MUST differ, so two Basecamp profiles cannot contend for one
  node's socket

#### Scenario: An empty profile name yields an unsuffixed socket name

- **WHEN** `resolveSocket` is given no configured socket, no `RAD_SOCKET`, a
  runtime directory `/run/user/1000`, an empty profile name and the home
  `/home/u/.radicle`
- **THEN** the result MUST be `/run/user/1000/radicle.sock`

#### Scenario: Without a runtime directory the socket falls back under the home

- **WHEN** `resolveSocket` is given no configured socket, no `RAD_SOCKET`, no
  runtime directory, the profile `alice` and the home `/home/u/.radicle`
- **THEN** the result MUST be `/home/u/.radicle/node/control.sock`

#### Scenario: Two homes yield two fallback sockets

- **WHEN** `resolveSocket` is called twice with no configured socket, no
  `RAD_SOCKET` and no runtime directory, once with the home `/home/a` and once
  with `/home/b`
- **THEN** the two results MUST differ
- **AND** each MUST be under the home it was given

#### Scenario: No socket source at all resolves to nothing

- **WHEN** `resolveSocket` is given no configured socket, no `RAD_SOCKET`, no
  runtime directory and no home
- **THEN** the result MUST be the empty string

### Requirement: The 108-byte sun_path cap constrains the socket alone

`resolvePaths` MUST reject a resolved socket path whose length plus a
terminating NUL exceeds the 108-byte `sun_path` capacity, by reporting a
non-empty problem: a path of exactly 107 bytes MUST be accepted and one of 108
bytes refused. The cap MUST NOT be applied to the home, and a home of any length
MUST NOT produce a problem. The check MUST run at resolution time rather than
at the point of connection.

#### Scenario: A path exactly at the cap is accepted and one byte over is refused

- **WHEN** `resolvePaths` is given a configured socket of 107 bytes
- **THEN** the reported problem MUST be empty
- **AND WHEN** `resolvePaths` is given a configured socket of 108 bytes
- **THEN** the reported problem MUST be non-empty

#### Scenario: A home past the cap is not a problem

- **WHEN** `resolvePaths` is given a configured home longer than 108 bytes and
  a runtime directory that yields a short socket
- **THEN** the resolved home MUST be that long home unchanged
- **AND** the reported problem MUST be empty

#### Scenario: A socket within the cap reports no problem

- **WHEN** `resolvePaths` is given the home `/home/u/.radicle`, no configured
  socket, no environment values, the runtime directory `/run/user/1000` and the
  profile `alice`
- **THEN** the resolved socket MUST be `/run/user/1000/radicle-alice.sock`
- **AND** the reported problem MUST be empty

### Requirement: The socket setting is validated against the same cap on write

Setting `radSocket` to a non-empty value whose length plus a terminating NUL
exceeds the same 108-byte capacity MUST be refused with an `{"error":"..."}`
reply naming the path, its byte length and the 107-byte limit, and the value
MUST NOT be persisted. An empty value MUST be accepted, meaning "resolve from
the environment".

#### Scenario: An over-long socket setting is refused

- **WHEN** `radSocket` is set to a path of 120 bytes
- **THEN** the reply MUST be an error naming that path, `120` and `107`
- **AND** a subsequent read of the settings MUST show the previous value

#### Scenario: A socket setting within the cap is accepted

- **WHEN** `radSocket` is set to `/run/user/1000/rad.sock`
- **THEN** the reply MUST NOT be an error
- **AND** a subsequent read of the settings MUST show that path

### Requirement: Embedded mode must never resolve the user's own Radicle home

When the selected mode is `embedded`, the home handed to path resolution MUST
be the embedded home, and the resolution MUST NOT be able to fall through to
`RAD_HOME` or `<user home>/.radicle`. An unresolvable embedded home MUST NOT be
passed through to resolution, which reads an empty configured home as "not
configured"; the mode MUST instead yield inert paths with an empty home.

#### Scenario: An unresolvable embedded home is inert rather than the user's home

- **WHEN** the mode is `embedded` and no XDG data home or user home is
  available to derive an embedded home from, while `RAD_HOME` names a real
  Radicle profile
- **THEN** the resolved home MUST be empty
- **AND** it MUST NOT be the `RAD_HOME` path
- **AND** local availability MUST be false

#### Scenario: A resolvable embedded home overrides RAD_HOME

- **WHEN** the mode is `embedded`, an XDG data home is available, and
  `RAD_HOME` names a different directory
- **THEN** the resolved home MUST be the embedded home derived from the XDG
  data home
- **AND** it MUST NOT be the `RAD_HOME` path

#### Scenario: Local mode does read the environment

- **WHEN** the mode is `local`, no `radHome` setting is configured, and
  `RAD_HOME` names a directory
- **THEN** the resolved home MUST be that `RAD_HOME` path
- **AND** it MUST differ from the embedded home the same environment would
  yield, so the two modes are distinguishable by their answer alone

#### Scenario: An unrecognised mode resolves nothing

- **WHEN** the persisted mode is a value this module does not map to a home
- **THEN** the resolved home MUST be empty
- **AND** it MUST NOT be the `RAD_HOME` path or `<user home>/.radicle`

### Requirement: The absent-profile sentence is chosen by the mode that asked

`NodePaths` MUST carry an optional `absentProfileReason`. When the home holds no
profile, that sentence MUST be returned verbatim in preference to either
default: the empty-home default as well as the missing-profile default.

The default sentence for a resolved but profile-less home MUST name the path
and advise installing Radicle and running `rad auth`.

#### Scenario: A supplied reason replaces the rad auth advice

- **WHEN** a store is built from paths with the home `/nowhere/embedded-home`
  and the supplied reason `no embedded identity yet — set one up here`
- **THEN** the unavailable reason MUST be exactly that sentence
- **AND** it MUST NOT mention `rad auth`

#### Scenario: Without a supplied reason the default advice stands

- **WHEN** a store is built from paths with the home `/nowhere/user-home` and
  no supplied reason
- **THEN** the unavailable reason MUST mention `rad auth`
- **AND** it MUST name `/nowhere/user-home`

#### Scenario: A supplied reason survives an empty home

- **WHEN** a store is built from paths with an empty home and a supplied reason
- **THEN** the unavailable reason MUST be that supplied sentence
- **AND** it MUST NOT be the `set RAD_HOME or HOME` default

#### Scenario: A supplied reason is unused when a profile is present

- **WHEN** a store is built from paths whose home holds a profile, with a
  supplied reason
- **THEN** availability MUST be true
- **AND** the unavailable reason MUST NOT contain the supplied sentence

#### Scenario: Embedded's sentence names the embedded home and its separateness

- **WHEN** the mode is `embedded`, an embedded home resolves, and it holds no
  identity
- **THEN** the unavailable reason MUST name the resolved embedded home
- **AND** it MUST state that this is a separate identity from any Radicle node
  the user already runs
- **AND** it MUST NOT advise running `rad auth`

#### Scenario: Embedded with nowhere to put a home says so

- **WHEN** the mode is `embedded` and no embedded home can be resolved
- **THEN** the unavailable reason MUST state that the module keeps the home
  under the Basecamp profile's data directory and that neither `XDG_DATA_HOME`
  nor `HOME` is set
- **AND** it MUST NOT advise setting `RAD_HOME`, which is the one variable
  embedded mode must never follow

### Requirement: The node is running only when its socket accepts a connection

A node MUST be reported as running only when the resolved socket path is
non-empty, is within the kernel's `sun_path` capacity, exists on disk, and
accepts a connection. A socket file that exists but accepts no connection MUST
NOT be reported as a running node. A store with no profile MUST report the node
as not running.

#### Scenario: A missing socket means the node is not running

- **WHEN** the resolved home holds a profile and no socket file exists at the
  resolved socket path
- **THEN** the node MUST be reported as not running

#### Scenario: An over-long socket path reports the node as not running

- **WHEN** the resolved socket path is at or beyond the kernel's `sun_path`
  capacity
- **THEN** the node MUST be reported as not running
