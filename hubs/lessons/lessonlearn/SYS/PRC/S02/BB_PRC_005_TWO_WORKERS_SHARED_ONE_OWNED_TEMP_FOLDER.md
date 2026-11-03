# BB-PRC-005 — lesson

id: BB-PRC-005
family: PRC
status: FIXED
source: gate runs 2026-11-04

## Who believed what

The leak fix (#67) was safe: every folder has one owner.

## What changed the belief

A test that failed only in the full gate, twice, named a project that
failed to load (BB-TG-017); the owner was per process, the users were
per window.

## Rule left behind

An owner's name must be as unique as its users: per window, per worker,
per test, not per process. A flake that names its cause is a bug report.
