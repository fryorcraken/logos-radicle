# Spec Delta

## Purpose

Define the repository sync: the download of every directory and file on the
selected branch into the view's cache, started and cancelled from one control,
and what a sync costs every other request the module serves.

## ADDED Requirements

### Requirement: A cancelled sync stops reaching its node

A sync is cancelled by activating the sync control while it runs. Its requests
are the tree and file reads it makes to fill the cache.

After a cancel, at most a fixed number of that sync's requests MUST reach the
node the sync reads from: the seed for `remote*` calls, the Radicle home for
`local*` ones. The bound MUST be the same for every mode, and MUST NOT depend
on the repository's size or on how much of the sync remained.

#### Scenario: A large sync cancelled early stops well short of its end

- **GIVEN** a sync of a repository holding many times more directories and
  files than the bound, cancelled after its first directory listing has been
  answered
- **WHEN** every request that is going to be served has been served
- **THEN** the number of the sync's requests that reached the node after the
  cancel MUST NOT exceed the bound
- **AND** it MUST be fewer than the number of requests the sync had still to
  make when it was cancelled

#### Scenario: The bound does not grow with the repository

- **GIVEN** two repositories, the second holding at least ten times as many
  files as the first and both holding more than the bound, each synced and
  cancelled once its first directory listing has been answered
- **WHEN** every request that is going to be served has been served
- **THEN** for each, the number of the sync's requests that reached the node
  after the cancel MUST NOT exceed the bound

#### Scenario: The bound holds against the local node

- **GIVEN** the mode in force derives the `local` prefix, and a sync of a
  repository holding many times more directories and files than the bound is
  cancelled after its first directory listing has been answered
- **WHEN** every request that is going to be served has been served
- **THEN** the number of the sync's `local*` requests that reached the Radicle
  home after the cancel MUST NOT exceed the bound

### Requirement: A request made after a cancel does not wait for the sync

A request made after a sync is cancelled, by any view, MUST NOT wait for the
cancelled sync's remaining work. Ahead of it, the module MUST NOT serve more of
that sync's requests than the requirement "A cancelled sync stops reaching its
node" allows to reach the node after the cancel.

#### Scenario: A branch listing after a cancel overtakes the sync's backlog

- **GIVEN** a node that answers one request at a time, and a sync cancelled
  while it still had many times more requests to make than the bound
- **WHEN** a `ListBranches` request is made immediately after the cancel
- **THEN** its reply MUST arrive after no more than the bound's worth of the
  sync's requests have been served
- **AND** its reply MUST carry the branches the node holds, not an error

#### Scenario: Cancel, then poll, in the order a user follows

- **GIVEN** a real seed, and a sync of a repository whose full download takes
  longer than a staleness poll is allowed to wait for its answer
- **WHEN** the user starts the sync, cancels it, and then a staleness poll is
  made for the same repository and branch
- **THEN** the poll's result MUST be shown within that wait
- **AND** the sync MUST still be reported as not running

### Requirement: A cancel settles every request it stops

Every request a cancelled sync made MUST settle, answered or refused, so that
once the requests still allowed to reach the node have been served, the view
MUST report itself not busy. A request that a cancel stops MUST NOT be reported
to the user as an error.

#### Scenario: The busy indicator clears after a cancel

- **GIVEN** a sync that has been cancelled while requests were outstanding
- **WHEN** the requests still allowed to reach the node have been served, and
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
- **THEN** the file's contents MUST be shown, matching what the node holds for
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
