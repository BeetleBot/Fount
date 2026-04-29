# Winget & Windows Portability: Static Linking

## The Issue
Currently, Windows builds of Fount are linked against the **Dynamic C Runtime (CRT)**. This means the resulting `.exe` depends on `vcruntime140.dll` (Visual C++ Redistributable) being present on the user's system.

If this DLL is missing (as it was in the Winget validation environment), the app fails to start with error `0xC0000135` (`STATUS_DLL_NOT_FOUND`).

## The Solution
Statically link the C runtime into the binary. This bundles the necessary C code directly into `fount.exe`, making it truly portable.

### Steps to Implement

#### 1. Update GitHub Actions
Modify `.github/workflows/release.yml` to set the `RUSTFLAGS` environment variable for the Windows build job.

Add `env: RUSTFLAGS: -C target-feature=+crt-static` to the `windows-installer` job:

```yaml
  windows-installer:
    name: Windows Installer (.msi)
    needs: [create-release]
    runs-on: windows-latest
    env:
      RUSTFLAGS: -C target-feature=+crt-static
    steps:
      - uses: actions/checkout@v4
      # ... rest of the steps
```

#### 2. Update Local Build (Optional)
If you build manually on Windows, you can create a `.cargo/config.toml` file to make this the default:

```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-feature=+crt-static"]
```

## Why this is better
- **Zero Dependencies**: Users don't need to install "Visual C++ Redistributables".
- **Easier Winget Approval**: No need to declare `Microsoft.VCRedist` in your manifest.
- **Portability**: The binary can be copied to any Windows machine and it will just work.
