## ADDED Requirements

### Requirement: A seed accepted by setRemoteSeed survives a restart

When `setRemoteSeed` accepts a seed, it MUST store that URL as the
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

### Requirement: A seed setRemoteSeed refuses changes nothing

`setRemoteSeed` MUST refuse with `{"error":"..."}` an empty URL, a URL that
`setSetting` would refuse for `remoteSeed`, a seed whose probe fails, and a
seed it cannot store. A refusal MUST leave both the seed in force and the
stored `remoteSeed` as they were before the call, and MUST NOT emit
`remoteSeedChanged`. A URL `setSetting` would refuse MUST be refused whether or
not that seed would answer a probe.

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
