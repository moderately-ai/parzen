# Releasing Parzen

Releases are prepared on a reviewed branch and published from an immutable annotated tag. Never
move or delete a release tag: repository rules protect every `v*` tag, and a source defect found
after tagging requires a new patch version.

## Prepare the candidate

1. Choose the version according to Cargo semantic versioning and update the root manifest, root
   lockfile, comparison-harness path requirement and lockfile, and current backend version labels.
2. Move the release notes from `Unreleased` to a dated changelog section and update its comparison
   links. Keep a new empty `Unreleased` section.
3. Add migration guidance for an MSRV, default-feature, public-API, or behavioral change.
4. Do not rewrite historical benchmark records, reports, or provenance to use the new version.
5. Confirm the unpublished comparison harness remains excluded from `cargo package`.

Run the repository checks documented in [CONTRIBUTING.md](CONTRIBUTING.md), plus:

```bash
./scripts/license-headers.sh check
cargo deny --all-features check
cargo +stable semver-checks check-release \
  --baseline-version <previous-version> \
  --release-type patch \
  --all-features
actionlint
cargo publish --dry-run --locked
```

Use patch-level semver checking when a release is intended to preserve the public API, even when
the selected pre-1.0 version increments the minor component. Inspect the complete output of
`cargo package --list --locked` and test the unpacked candidate with default, all, and no default
features. Record the inventory and SHA-256 checksum in the pull request.

The comparison harness remains manual. Run its tests, Clippy, release and profiling builds, smoke
suite, schema parser, and deterministic report regeneration. A documentation-only or
version-only release change does not require repeating a complete performance envelope.

## Review and integrate

Open a pull request to `main` and require all protected checks and the configured approving review.
The pull request must identify API, MSRV, default-feature, quality, package, and performance effects
separately.

If a release line must preserve existing commit or tag ancestry, use a non-force administrative
fast-forward only after explicit approval:

1. Fetch `origin/main` and require it to be an ancestor of the reviewed candidate.
2. If `main` advanced, merge it into the candidate; never rebase commits referenced by a release
   tag. Repeat review and CI.
3. Save the complete branch-protection configuration.
4. Install a trap that restores administrator enforcement.
5. Temporarily disable only administrator enforcement.
6. Push `<candidate>:refs/heads/main` without `--force`.
7. Restore enforcement immediately and verify the full protection configuration and remote SHA.

Do not update `main` merely because checks pass. Integration requires explicit maintainer approval.

## Verify the tag candidate

From a clean checkout of the exact remote `main` commit:

1. Recheck version, changelog, package inventory, tests, and registry availability.
2. Create `v<version>` as an annotated tag and push it once.
3. Wait for the tag-triggered `release-crates` verification job.
4. Download the `crates-io-candidate` artifact.
5. Verify its checksum, inventory, contents, and `.cargo_vcs_info.json` commit.
6. Test the unpacked crate before authorizing publication.

A tag push verifies and stores a candidate but does not publish it.

## Publish

Publication requires a second explicit approval after tag verification. Dispatch
`release-crates` on the exact tag with `publish=true`, then approve the `crates-io` environment.
The workflow requires the rebuilt candidate, verified artifact, and crates.io index to have the
same checksum. It creates or updates the GitHub release from the matching changelog section and
attaches the candidate, inventory, and checksum.

After publication, verify:

- crates.io reports the expected version, MSRV, features, checksum, and non-yanked state;
- default, selected-feature, and no-default-feature consumer builds resolve from crates.io;
- docs.rs successfully builds the tagged version;
- the GitHub tag resolves to the exact `main` commit; and
- the GitHub release notes and attached artifacts match the verified candidate.

Keep the release branch until these checks pass.

## Recovery

- A transient verification or publication job may be rerun against the same immutable tag.
- If the version is already published, the workflow succeeds only when its registry checksum
  matches the verified candidate.
- If a source defect is found after tagging but before publication, do not move the tag. Prepare the
  next patch version.
- If crates.io publication succeeds but GitHub release creation fails, rerun the idempotent release
  job or create the release from the verified artifacts. Do not republish.
- Do not yank a published version without a separately reviewed incident decision.
