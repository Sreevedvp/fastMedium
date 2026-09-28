# Installed Fast UI components

This is a consumer-owned snapshot of Fast UI 0.1.0, installed from the separate
`dioxus-complib` project using `dx components add`. The canonical library lives
at `/Users/sreevedvp/Documents/ChatGPT/dioxus-complib`; it is not nested inside
FastMedium. FastMedium builds without that checkout.

To update, install the desired components into a temporary Dioxus app, compare
against these files, and merge local edits. Never force-overwrite a customized
component. Use `dx components add theme button field table dialog --path
/path/to/dioxus-complib --module-path src/ui` for a fresh installation.
