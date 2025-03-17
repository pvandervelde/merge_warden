//! # Validation Checks
//!
//! This module contains the validation checks that are performed on pull requests.
//!
//! The checks are organized into submodules:
//! - `title`: Validates that PR titles follow the Conventional Commits format
//! - `work_item`: Validates that PR descriptions reference a work item or issue
//!
//! These checks are used by the `MergeWarden` to determine if a PR is valid
//! and can be merged.

pub mod title;
pub mod work_item;
