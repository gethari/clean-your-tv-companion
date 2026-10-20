# Local launcher protocol candidate

`launcher-protocol.patch` contains four commits based on owner runtime source
`e14d13ad096929ef033d3afd4e6c3fabca2953e8`. Apply it to that base in a separate checkout:

```sh
git am /path/to/launcher-protocol.patch
```

The resulting source commit is recorded in `launcher-release.json`. The isolated checkout used for
verification is /tmp/cytv-launcher-owner on branch feat/companion-owner-runtime. The original owner
working tree and its uncommitted customizations were not edited. Existing wallpaper and remote-key
source additions are included so this patch preserves the installed runtime. No theme configuration,
API credentials, signing keys or private device addresses are included.

Earlier installed candidate verification: 69 JVM tests plus actual Android provider calls and pipe uploads on the existing TV.
A separate unprivileged test app was rejected by Binder caller checks and then removed. Protocol
smoke tests staged/discarded media without activating it; existing theme bytes, HOME and disabled
packages remained unchanged. This is a local candidate, not an upstream approval or published release.
See docs/LAUNCHER_PROTOCOL.md and docs/REVIEW.md in the companion for the contract and limits.

APK signing is installation-specific. Verify the candidate certificate against the existing app;
never uninstall a user's launcher to bypass a signature mismatch. APK/data downgrades are not a
promised restoration path. Private update backups are kept separately from this source repository.

## Source checkpoint

The source pin includes offline-tested playback and unfinished scheduling work. Scheduling is not
advertised in capabilities. `installed_candidate_source_commit` and `installed_candidate_apk_sha256`
identify the earlier physical test candidate; they do not describe an APK built from the latest source.
No new source was installed after the user prohibited live TV testing. See [HANDOFF.md](../HANDOFF.md)
for continuation and verification requirements. The patch applies all source commits from the pinned
owner base, including the checkpoint, without modifying the original user working tree.
