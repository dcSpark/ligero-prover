# Portable binaries packaging

This directory contains helper scripts used by `../build-portable-binaries.sh`.

## Output layout

The main script produces:

```
ligero/
  bins/
    linux-amd64/
      bin/
      lib/
    linux-arm64/
      bin/
      lib/
    macos-arm64/
      bin/
      lib/
  shader/
    *.wgsl
```

And a single tarball `ligero-bins.tar.gz` containing the `ligero/` folder.

## Boost Version Requirement

**IMPORTANT:** All platforms must use Boost >= 1.84 (archive version 19) for cross-platform proof compatibility.

- **Linux:** Boost 1.89 is built from source during the build process
- **macOS:** Requires Homebrew Boost >= 1.84 (run `brew upgrade boost` if needed)

The portable binary archives use a fixed archive version (19) defined in `include/util/boost/portable_binary_archive.hpp`. This ensures that proofs generated on any platform can be verified on any other platform.

If you see "boost.archive: unsupported version" errors, it means the Boost versions are mismatched.

## Notes

- Linux builds run in Docker and include an RPATH of `$ORIGIN/../lib` so binaries prefer the bundled `lib/` directory.
- macOS builds run natively (Apple Silicon) and add `@executable_path/../lib` as an rpath so binaries prefer the bundled `lib/` directory.


