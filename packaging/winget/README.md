WinGet Packaging for Rivet

Overview
- We publish a Windows ZIP in releases that contains:
  - `rivet.exe`
  - `rivet.ps1` (PowerShell completion)
- A GitHub Actions workflow (`.github/workflows/winget.yml`) uses `microsoft/winget-create` to open a PR to `microsoft/winget-pkgs` when a release is published.

Prerequisites
- Create a GitHub personal access token (classic) with `public_repo` scope, save it in repo secrets as `WINGET_TOKEN`.
- Decide on a WinGet Package Identifier (e.g., `Szabadkai.Rivet`). Update `WINGET_IDENTIFIER` in the workflow if needed.

How it works
1) On release publish, the workflow computes the version and the ZIP asset URL (`rivet-vX.Y.Z-windows-x86_64.zip`).
2) `wingetcreate` generates or updates the manifest using the ZIP (InstallerType: zip) and submits a PR to `winget-pkgs`.
3) The manifest can map the nested `rivet.exe` as a portable command alias `rivet` (the action auto-detects portable exe; if needed, we can refine manifests in follow-up PRs).

Manual testing
- After a PR merges, test on Windows:
  - `winget install Szabadkai.Rivet`
  - Ensure `rivet` is on PATH and `rivet.ps1` is present in the install directory.
  - Add completion to PowerShell profile if desired:
    - `New-Item -Path $PROFILE -ItemType File -Force | Out-Null`
    - `Add-Content -Path $PROFILE -Value ". '$env:LOCALAPPDATA\\Microsoft\\WinGet\\Packages\\Szabadkai.Rivet_*\\rivet.ps1'"`

Notes
- We still recommend publishing a small module to PowerShell Gallery for the smoothest autoload experience; this ZIP bundling keeps things self-contained for now.
- If the winget bot cannot infer nested files properly, we can check in curated manifests under `packaging/winget/` and switch the workflow to use them.

