# 🗺️ Frontier Roadmap

A strategic overview of the development phases for **Frontier**, from initial setup to final static binary distribution.

---

## 🏆 Completed

* **CLI Route Mapping:** Finalize logic for `.\frontier`, `.\back`, and `.\front` to ensure correct directory scoping.
* **Project Scaffolding:** Standardize the `app/frontend/` and `app/backend/` hierarchy to ensure the Engine finds `index.html` and source files.
* **Metadata Implementation:** Configure `frontier.toml` to handle `[app]` metadata (Name, Version, Copyright) for Windows file properties.
* **Window Meta-Tag Parser:** Implement the engine's ability to read `frontier-width`, `frontier-height`, and `frontier-title` from the HTML `<head>`.
* **Math Formula Resolver:** Build the logic to calculate `screen_w`, `win_w`, and center the window via `frontier-x/y` formulas.
* **Module Manifest System:** Create the initial `manifest.toml` structure for `modules/` to define how `.c`, `.py`, and other extensions are handled.
* **Persistence Layer (ID/Save):** Setup the `%LOCALAPPDATA%` storage logic based on the `frontier-id` meta tag.
* **IPC Bridge Foundation:** Establish the `window.ipc.postMessage` communication link between the Webview and the Backend triggers.
* **Icon Handling:** Integrate the logic to pull the `.ico` from the path defined in `frontier.toml` for the final executable.
* **Dependency Audit:** Verify that the `.frontier/` folder (Rust/Build System) is correctly isolated from the user source code.

---

## 🏗️ Upcoming Development

* **Cross-Platform Linux Support:** Implement a Linux-compatible build pipeline, ensuring core engine functionality and Webview bindings across distributions using a unified source code.
* **Module Distribution System:** Develop a system for remote module acquisition via Git or Zip, enabling distribution of assets beyond `manifest.toml` with integrated versioning and update checks.
* **Multi-Extension & Directory-Based Modules:** * **Unified Manifests:** Update `manifest.toml` to support multiple extensions (e.g., `extension = ["c", "cpp"]`) within a single file.
    * **Directory Scoping:** Implement `dir_extension` for folder-based modules, allowing for specialized lifecycle hooks like `[dir_dev]` and `[dir_build]` for complex compilation or asset processing.
* **Single Static Binary Distribution:** Develop a simplified build process to bundle all assets, backend logic, and the engine into a single, standalone executable for easy portability and deployment.

---

## ⚠️ Maintenance & Stability

* **System Integrity:** No critical bugs or regressions identified. Continuous monitoring of IPC bridge stability and metadata parsing is ongoing.