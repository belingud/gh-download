## Why

Large directory downloads currently fetch files one by one after enumeration, which makes the CLI noticeably slow on repositories with many small files or high per-request latency. We need bounded concurrent downloads so directory transfers finish faster without changing the existing GitHub metadata detection flow, proxy boundaries, or streaming-to-disk behavior.

## What Changes

- Add bounded concurrent file downloads for directory targets so multiple files can be transferred and written in parallel after directory enumeration completes.
- Add a CLI option to control directory download concurrency with a safe default and validation for invalid values.
- Keep file downloads single-target and keep all download workers using the same authentication, prefix-proxy, and raw-download fallback rules that exist today.
- Update progress and completion output so concurrent directory downloads remain understandable in both English and Chinese.
- Update specs and bilingual documentation to describe concurrent directory downloads, the concurrency option, and the unchanged proxy and disk-streaming constraints.

## Capabilities

### New Capabilities
- `download-concurrency`: Configure and enforce bounded concurrent file downloads for directory transfers.

### Modified Capabilities
- `github-path-download`: Extend directory download behavior and progress reporting to cover concurrent file transfers while preserving relative-path writes, target resolution, metadata-first detection, and proxy/authentication boundaries.

## Impact

- Affected code: `src/cli.rs`, `src/cli/types.rs`, `src/cli/help.rs`, `src/download.rs`, `src/download/transport.rs`, `src/output.rs`, and related tests
- Affected docs: `README.md`, `README.zh.md`
- Affected specs: `openspec/specs/github-path-download/spec.md` and new `download-concurrency` capability spec
- Dependencies/systems: may require a small worker-pool/concurrency helper, but must continue streaming file bytes directly to disk
