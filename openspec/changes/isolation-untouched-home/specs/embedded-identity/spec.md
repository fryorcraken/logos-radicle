# Spec Delta

## ADDED Requirements

### Requirement: Creating an identity leaves every other home untouched

Where neither home's path is a prefix of the other's, creating an identity into
one home through the identity-creation backend MUST NOT add or remove any entry
under the other home, MUST NOT change the bytes of any file under it, and MUST
NOT create the other home's path when it does not exist.

#### Scenario: Creating into home A leaves home B's profile as it was

- **GIVEN** an identity has been created into home B
- **AND** every entry under home B has been recorded by its path relative to
  home B, with the bytes of each file
- **AND** the record holds `keys/radicle`, `keys/radicle.pub` and
  `config.json`
- **WHEN** an identity is then created into home A, where neither path is a
  prefix of the other
- **THEN** the creation into home A MUST report a non-empty `nodeId`
- **AND** the entries under home B MUST be exactly the recorded ones, with
  none added and none removed
- **AND** every file under home B MUST be byte-for-byte what was recorded

#### Scenario: Creating into home B does not bring home A into being

- **GIVEN** home A's path does not exist
- **WHEN** an identity is created into home B, where neither path is a prefix
  of the other
- **THEN** home A's path MUST still not exist
