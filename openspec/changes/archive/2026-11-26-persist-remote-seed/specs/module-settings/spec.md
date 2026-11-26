## ADDED Requirements

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

## MODIFIED Requirements

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
