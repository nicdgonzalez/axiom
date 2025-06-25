// Axiom is a command-line tool for managing PaperMC Minecraft servers.
// Copyright (C) 2025 nicdgonzalez
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]
#![doc(test(attr(deny(dead_code))))]

pub mod manifest;
pub mod package;
pub mod paper;
pub mod varint;

pub use manifest::{Manifest, ManifestError};
pub use package::Package;
