## MODIFIED Requirements

### Requirement: Every mode with an identity shows it in the header

The header MUST display the DID the backend reports for the mode in force,
whenever the mode in force has one and the reported DID is non-empty, in
`embedded` exactly as in `local`.

The DID MUST be displayed in full, including its `did:key:` prefix, and MUST be
the one the backend reported rather than a truncation or a placeholder.

Where the mode has no identity, as in `explore`, or its reported DID is empty,
nothing MUST be displayed: no blank slot and no placeholder.

#### Scenario: Embedded shows its DID as Local does

- **GIVEN** a header in `local` whose backend reports a distinctive DID
- **THEN** that DID MUST be displayed in full
- **AND WHEN** the header is in `embedded` and the backend reports a different
  distinctive DID
- **THEN** the second DID MUST be displayed in full
- **AND** the first MUST NOT be displayed

#### Scenario: A mode with no identity displays nothing

- **GIVEN** a header in `explore`, whose backend reports an empty DID
- **THEN** no identity MUST be displayed
- **AND WHEN** the header is in `embedded` with an empty reported DID, which is
  the state before an identity has been created
- **THEN** no identity MUST be displayed
- **AND WHEN** the same header is told a non-empty DID with nothing else changed
- **THEN** that DID MUST be displayed

#### Scenario: The displayed identity follows the reported one

- **GIVEN** a header in `embedded` told a distinctive DID
- **THEN** that DID MUST be displayed
- **AND WHEN** the header is told a second, different DID
- **THEN** the second MUST be displayed and the first MUST NOT

### Requirement: The header identity is copied by taking it

Taking the displayed identity MUST put the DID on the system clipboard, in
`embedded` as in `local`, and MUST confirm only once it is there: the
confirmation MUST NOT be shown for a copy that did not reach the clipboard.

What is copied MUST be the full DID the backend reported, whatever the header is
displaying, so a display shortened to fit a narrow window still copies the
whole value.

#### Scenario: Taking the identity copies it in Embedded

- **GIVEN** a header in `embedded` displaying a distinctive DID
- **WHEN** the identity is taken
- **THEN** the clipboard MUST hold that DID
- **AND** the confirmation MUST be shown

#### Scenario: The confirmation is not shown for a copy that did not land

- **GIVEN** a header in `embedded` displaying a DID, where the clipboard does
  not retain what is written to it
- **WHEN** the identity is taken
- **THEN** the confirmation MUST NOT be shown

#### Scenario: The whole DID is copied whatever is displayed

- **GIVEN** a header in `embedded` displaying a DID shortened to fit
- **WHEN** the identity is taken
- **THEN** the clipboard MUST hold the full reported DID

## ADDED Requirements

### Requirement: One rule decides the header identity for every mode

Whether the header displays an identity MUST be decided by one rule covering
every mode, keyed on whether the mode in force has an identity, and MUST NOT be
decided by a list of modes that each have the display switched on.

A mode has an identity exactly when it derives the `local` method prefix, as
`source-modes`' "The method prefix is derived from the mode" decides it. A mode
that derives `remote` has none. Whether that identity exists yet is the
reported DID, a separate question.

#### Scenario: The identity is shown in exactly the modes that derive the local prefix

- **GIVEN** a header whose backend reports a distinctive DID
- **WHEN** the mode is `local`, and then `embedded`
- **THEN** that DID MUST be displayed in each
- **AND WHEN** the mode is `explore`, and then a string the view does not
  recognise
- **THEN** no identity MUST be displayed in either
