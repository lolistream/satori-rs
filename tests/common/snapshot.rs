//! Helpers around the snapshot directory at `snapshots/`.

/// Build a snapshot filename in the convention originally used by
/// `jest-image-snapshot`. Names look like:
///
///   `<file>-test-tsx-test-<file>-test-tsx-<describe>-<it>-<n>-snap.png`
///
/// We accept the already-formed file-stem `<describe>-<it>-<n>` and compose
/// the rest.
pub fn snap_name(file_slug: &str, key: &str) -> String {
    format!(
        "{file_slug}-test-tsx-test-{file_slug}-test-tsx-{key}-snap.png",
        file_slug = file_slug,
        key = key,
    )
}
