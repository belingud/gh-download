## Context

`gh-download` currently resolves a GitHub path through the Contents API, then downloads files either from the returned `download_url` or from the same Contents API endpoint with `Accept: application/vnd.github.raw`. Recent proxy changes mixed three separate concerns: the user's ambient system proxy, URL-prefix proxies such as `gh-proxy`, and GitHub metadata API access. That created ambiguous logs, incorrect fallback behavior for metadata JSON endpoints, and no clean way for a user to opt into prefix-first raw downloads.

The requested behavior is explicit:
- show short failure reasons instead of vague "direct URL unavailable" messaging
- support prefix proxy modes without probing or productizing system proxy state
- provide a permanent debug mode that prints constructed URLs and strategy choices when explicitly enabled

## Goals / Non-Goals

**Goals:**
- Introduce an explicit prefix proxy mode with `direct`, `fallback`, and `prefer`
- Restrict URL-prefix proxy usage to raw file download URLs only
- Keep GitHub metadata API requests on their normal path
- Remove system-proxy detection from user-facing output
- Add a permanent opt-in debug mode for generated metadata URLs, resolved download URLs, prefix URLs, and strategy choices
- Tighten warning/error output so users see concise, actionable reasons

**Non-Goals:**
- Changing how `reqwest` honors standard proxy environment variables internally
- Introducing region-aware defaults or auto-detection of proxy preferences
- Adding a persistent config file format in this change

## Decisions

1. Prefix proxy mode is an explicit CLI/env policy, not inferred from the runtime environment.
Rationale: this preserves an internationally neutral default while allowing local opt-in. The mode should be resolved in `cli.rs` so the runner receives a simple enum-like configuration.
Alternatives considered:
- Infer from locale, timezone, or network errors: rejected because it is unstable and surprising
- Reuse `proxy_base` presence as the only switch: rejected because it cannot express "prefer prefix first"

2. URL-prefix proxying applies only to `download_url`-style raw file downloads.
Rationale: `gh-proxy` usage matches raw GitHub asset URLs such as `raw.githubusercontent.com/...`; it is not a reliable transport for GitHub Contents API JSON responses.
Alternatives considered:
- Continue retrying `api.github.com/repos/.../contents/...` through prefix proxies: rejected because it breaks API semantics and caused JSON decode failures
- Prefix-proxy both `download_url` and Raw API responses: rejected because the Raw API path is still an API endpoint, not the canonical gh-proxy input shape

3. URL and strategy diagnostics are exposed through a permanent `--debug` mode.
Rationale: debugging request construction is a recurring development need, not a one-off migration step. A stable opt-in mode avoids repeated temporary prints and keeps normal CLI output clean.
Alternatives considered:
- Temporary ad hoc print statements: rejected because they pollute implementation and are easy to forget
- A narrower `--debug-urls` flag: rejected because the user wants broader flow-level debugging, not only URLs

4. System proxy usage remains an HTTP-client concern and is not surfaced as product-level startup messaging.
Rationale: the user asked to stop actively probing proxy state. `reqwest` can continue honoring ambient environment variables without the CLI treating them as part of the product contract.
Alternatives considered:
- Keep startup proxy detection output: rejected because it creates noise and couples behavior to environment inspection

5. User-facing warnings should name the failing stage briefly.
Rationale: the current "direct file URL unavailable" wording is too vague. The warning should say whether the failure occurred in direct raw download, prefix retry, or Raw API fallback, without dumping transport internals by default.
Alternatives considered:
- Print full low-level error chains unconditionally: rejected because it is noisy for normal usage

## Risks / Trade-offs

- [Debug output may be noisy] → Keep it opt-in, send it to stderr, and limit it to flow-relevant fields
- [Adding a new prefix mode expands CLI surface] → Keep the mode set small and document env/flag precedence clearly
- [Users may expect prefix mode to affect directory metadata reads] → State clearly in docs/spec that prefix mode only applies to raw file downloads
- [Changing fallback order can alter network behavior in existing setups] → Keep default mode as `direct` and require explicit opt-in for `prefer`
