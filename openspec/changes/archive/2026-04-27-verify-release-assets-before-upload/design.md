## Context

The release workflow builds each target, stages a package directory under `dist/`, compresses it, and then uploads the archive with `actions/upload-artifact`. The packaging steps already copy the binary and repository metadata files, but the workflow does not inspect the archive before upload.

This change should stay inside the release workflow and main release spec. It should not change build targets, package names, checksum generation, or release publishing.

## Goals / Non-Goals

**Goals:**

- Fail the build job before artifact upload if an archive misses the expected binary, `README.md`, `README.zh.md`, or `LICENSE`.
- Validate both Unix `.tar.gz` archives and Windows `.zip` archives.
- Run the packaged binary with `--version` only when the current runner can execute the target binary.

**Non-Goals:**

- Add signing, provenance, or attestations.
- Add new release targets.
- Change archive names or release body contents.

## Decisions

- Add validation steps directly after the package steps and before upload.
  - Rationale: this is the last point where each matrix job still knows the target triple and can inspect its own archive.
  - Alternative considered: validate in the publish job after downloading artifacts. That would check archives later, but it cannot easily run platform-native binaries from each matrix job.

- Use platform-native shell tools already available on GitHub-hosted runners.
  - Rationale: `tar` works for Unix archives, PowerShell `Expand-Archive` works for Windows archives, and no new action or dependency is needed.
  - Alternative considered: add a script file in the repository. The checks are small and release-workflow-specific, so inline steps keep the project simpler.

- Run `--version` for non-cross native builds and skip it for cross-built Linux targets.
  - Rationale: macOS runners can execute their matching macOS binary, and Windows can execute the Windows binary. Linux musl targets are built through `cross`, including an ARM64 target that cannot run on the x86_64 runner.
  - Alternative considered: use emulation for cross-built targets. That adds maintenance cost without matching this small-tool release goal.

## Risks / Trade-offs

- Archive inspection relies on the package directory naming convention -> derive the package name from the same version and target values used during packaging.
- A native binary might fail because of runner differences -> keep the executable check limited to targets that the runner is expected to run.
