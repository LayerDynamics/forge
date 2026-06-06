// forge-smelt embedded-app bootstrap.
//
// This module is the stable entry point of a *smelted* (ahead-of-time compiled)
// Forge app. When an app is embedded into a standalone binary (Depth 2), the
// runtime loads this single, well-known module, which in turn imports the app's
// compiled entry (`main.js`). Pinning the boot module name lets the runtime
// start any app without knowing its internal entry filename.
//
// It is transpiled to `init.js` at build time (see build.rs) and embedded as
// `forge_smelt::BOOTSTRAP_SHIM`; `forge_smelt::build::embed` writes it alongside
// the compiled module tree.
import "./main.js";
