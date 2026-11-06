# BB-TG-021 — A sum of RSS overstated CI memory

id: BB-TG-021
status: FIXED
for: none
date: 2026-11-06

## Gap

The first CI run with parallel workers reported a memory peak of 11.7 GB
on a 7 GB runner. The sampler added up every process's RSS, which counts
shared pages (libraries, the dotnet runtime) once per process. The number
could not tell whether the worker count was safe.

## Workaround

`.github/bb_scripts/ci_tests.sh` reads memory as macOS counts it in use:
`vm_stat` active + wired + compressor pages, plus swap from
`vm.swapusage`. PR #92's run: 7.6 GB of 14 GB, swap 0, 10 workers.
