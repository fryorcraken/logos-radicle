# Spec Delta

## ADDED Requirements

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

## MODIFIED Requirements

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
