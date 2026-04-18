# Fount Distribution Plan: Package Managers

This document outlines the strategy for expanding Fount's reach through native package managers on Windows, macOS, and Linux.

## 1. Windows: Scoop & Winget

### Scoop
Scoop is the preferred command-line installer for Windows developers. It avoids the "Smart App Control" friction often associated with MSI installers.

- **Requirements**:
    - A portable `fount-windows-x64.zip` containing `fount.exe` must be added to GitHub Releases.
- **Implementation**:
    - [ ] Update `.github/workflows/release.yml` to re-enable the Windows build matrix (binary only).
    - [ ] Create a `fount.json` manifest.
    - [ ] (Optional) Create a `BeetleBot/scoop-bucket` repository or submit to the `extras` bucket.

### Winget
Winget is the official Windows Package Manager. It is built-in and familiar to "normie" users.

- **Requirements**:
    - Same portable `.zip` or a standalone `.exe`.
- **Implementation**:
    - [ ] Use `wingetcreate` to generate a manifest once the binary is released.
    - [ ] Submit the manifest to the [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs) repository.

---

## 2. macOS & Linux: Homebrew

Homebrew is the standard for macOS and is widely used on Linux (Linuxbrew). It provides a unified experience for non-Windows users.

- **Requirements**:
    - Existing `.tar.gz` artifacts in GitHub Releases (already handled by current pipeline).
- **Implementation**:
    - [ ] Create a `fount.rb` Formula.
    - [ ] Create a dedicated Tap repository: `BeetleBot/homebrew-tap`.
    - [ ] Action: `brew tap BeetleBot/tap` followed by `brew install fount`.

---

## 3. The "Common" Backend: GitHub Actions Update

To support all the above, we need to ensure our release pipeline produces consistent artifacts.

### Proposed Matrix for `release.yml`:
| Platform | Target | Artifact |
| :--- | :--- | :--- |
| **Windows** | `x86_64-pc-windows-msvc` | `fount-vX.Y.Z-windows-x64.zip` |
| **macOS** | `universal-apple-darwin` | `fount-vX.Y.Z-macos-universal.tar.gz` |
| **Linux** | `x86_64-unknown-linux-musl` | `fount-vX.Y.Z-linux-x64.tar.gz` |
| **Linux ARM** | `aarch64-unknown-linux-musl` | `fount-vX.Y.Z-linux-arm64.tar.gz` |

---

## Next Steps
1. **Re-enable Windows Binaries**: Update the workflow to build the ZIP but skip the MSI.
2. **Setup Tap/Bucket**: Create the helper repositories for Homebrew and Scoop.
3. **Automate Manifests**: Ideally, add a step to the release workflow that automatically updates the Homebrew formula version.
