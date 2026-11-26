# Spec Delta

## Purpose

Define the repository sync: the download of every directory and file on the
selected branch into the view's cache, started and cancelled from one control,
and what a sync costs every other request the module serves. Which prefix a
view's calls carry is `source-modes`' requirement "The method prefix is derived
from the mode"; this capability says only where the sync control follows it.

## ADDED Requirements

### Requirement: The sync control is the seed's

The sync control MUST be shown while the method prefix derived from the mode is
`remote`, and MUST NOT be shown while it is `local`. A sync is therefore made of
`remote*` calls, and the node it reads from is the seed.

#### Scenario: A repository read through the seed offers the sync control

- **GIVEN** a repository open while the derived prefix is `remote`
- **THEN** the sync control MUST be shown

#### Scenario: A repository read through a local node offers no sync control

- **GIVEN** a repository open while the derived prefix is `local`
- **THEN** the sync control MUST NOT be shown

### Requirement: A running sync keeps at most 4 requests outstanding

A sync's requests are the tree and file reads it makes to fill the cache. A
request is outstanding from when it is made until its reply or refusal arrives.

While a sync runs, it MUST NOT have more than 4 of its requests outstanding at
once, however large the repository.

#### Scenario: A large sync never has more than 4 requests outstanding

- **GIVEN** a repository holding many more than 4 directories and files, and a
  node that answers one request at a time
- **WHEN** the repository is synced and the node answers every request
- **THEN** the sync MUST NOT at any point have had more than 4 of its requests
  outstanding
- **AND** the sync MUST complete, marking the repository as synced

#### Scenario: A late reply from a cancelled sync does not widen the next one

- **GIVEN** a sync cancelled while some of its requests were outstanding, and a
  new sync of the same repository and branch started before they are answered
- **WHEN** the cancelled sync's requests are answered, one at a time, while the
  new sync runs
- **THEN** the new sync MUST NOT at any point have more than 4 of its own
  requests outstanding

### Requirement: A cancelled sync stops reaching its node

A sync is cancelled by activating the sync control while it runs.

After a cancel, at most 4 of that sync's requests MUST reach the seed, however
large the repository and however much of the sync remained.

#### Scenario: A large sync cancelled early stops well short of its end

- **GIVEN** a sync of a repository holding many more than 4 directories and
  files, cancelled after its first directory listing has been answered
- **WHEN** every request that is going to be served has been served
- **THEN** the number of the sync's requests that reached the seed after the
  cancel MUST NOT exceed 4
- **AND** it MUST be fewer than the number of requests the sync had still to
  make when it was cancelled

#### Scenario: The bound does not grow with the repository

- **GIVEN** two repositories, the second holding at least ten times as many
  files as the first and both holding more than 4 directories and files, each
  synced and cancelled once its first directory listing has been answered
- **WHEN** every request that is going to be served has been served
- **THEN** for each, the number of the sync's requests that reached the seed
  after the cancel MUST NOT exceed 4

### Requirement: A request made after a cancel does not wait for the sync

A request made after a sync is cancelled, by any view, MUST NOT wait for the
cancelled sync's remaining work. Ahead of it, the module MUST NOT serve more
than 4 of that sync's requests.

#### Scenario: A branch listing after a cancel overtakes the sync's backlog

- **GIVEN** a node that answers one request at a time, and a sync cancelled
  while it still had many more than 4 requests to make
- **WHEN** a `ListBranches` request is made immediately after the cancel
- **THEN** its reply MUST arrive after no more than 4 of the sync's requests
  have been served
- **AND** its reply MUST carry the branches the node holds, not an error

#### Scenario: Cancel, then poll, in the order a user follows

- **GIVEN** a real seed; a wait longer than that seed takes to serve 4 of the
  sync's requests and the poll's own; and a sync cancelled while it still had
  more requests to make than the seed can serve within that wait
- **WHEN** the user starts the sync, cancels it, and then a staleness poll is
  made for the same repository and branch
- **THEN** the poll's result MUST be shown within that wait
- **AND** the sync MUST still be reported as not running

### Requirement: A cancel settles every request it stops

Every request a cancelled sync made MUST settle, answered or refused, so that
once the requests still allowed to reach the seed have been served, the view
MUST report itself not busy. A request that a cancel stops MUST NOT be reported
to the user as an error.

#### Scenario: The busy indicator clears after a cancel

- **GIVEN** a sync that has been cancelled while requests were outstanding
- **WHEN** the requests still allowed to reach the seed have been served, and
  nothing else has been asked
- **THEN** the view's busy indicator MUST be off

#### Scenario: A cancel raises no error

- **GIVEN** no error is shown before a sync is started
- **WHEN** the sync is cancelled and its stopped requests have settled
- **THEN** no error MUST be shown

### Requirement: A cancel stops only the sync

A cancel MUST NOT stop or refuse a request the cancelled sync did not make,
whenever that request was made. A sync started after a cancel MUST NOT
have any of its requests stopped by that earlier cancel, and a reply to a
request of the cancelled sync MUST NOT change the later sync's progress.

#### Scenario: A file opened during the sync still opens

- **GIVEN** a running sync, and a file the sync has not yet fetched
- **WHEN** the user opens that file, and the sync is cancelled before the
  file's contents arrive
- **THEN** the file's contents MUST be shown, matching what the seed holds for
  that path

#### Scenario: A sync started after a cancel downloads everything

- **GIVEN** a sync that was cancelled before it finished
- **WHEN** a new sync of the same repository and branch is started, and the
  node answers every request
- **THEN** every directory and file on that branch MUST be requested or already
  cached
- **AND** the new sync MUST complete, marking the repository as synced

#### Scenario: A late reply from the cancelled sync is ignored

- **GIVEN** a sync cancelled while one of its requests was being served, and a
  new sync started after the cancel
- **WHEN** the reply to the cancelled sync's request arrives
- **THEN** the new sync's progress MUST be unchanged by it

### Requirement: What a cancel leaves behind

A cancel MUST end the sync's in-progress state at once, without waiting for any
reply. A cancelled sync MUST NOT mark the repository as synced, and MUST NOT
change the commit recorded as the one the repository was last synced to.

#### Scenario: The sync stops showing progress at once

- **GIVEN** a running sync with requests outstanding
- **WHEN** it is cancelled, before any further reply arrives
- **THEN** the sync MUST be reported as not running
- **AND** the sync control MUST show its idle wording rather than a percentage

#### Scenario: A cancelled first sync still offers the first download

- **GIVEN** a repository that has never completed a sync
- **WHEN** a sync of it is started and then cancelled
- **THEN** the repository MUST NOT be reported as synced
- **AND** the sync control MUST read `Download All`

#### Scenario: The last synced commit survives a cancel

- **GIVEN** a repository whose last completed sync recorded a commit, and whose
  branch head has since moved to a different commit
- **WHEN** a sync of it is started and then cancelled
- **THEN** the commit recorded as last synced MUST be the one recorded before
  the sync started
