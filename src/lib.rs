/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Visibly pushdown automata.

#![deny(warnings)]
#![allow(unknown_lints)]
#![warn(
    clippy::all,
    clippy::missing_docs_in_private_items,
    clippy::nursery,
    clippy::pedantic,
    clippy::perf,
    clippy::restriction,
    clippy::cargo,
    elided_lifetimes_in_paths,
    missing_docs,
    rustdoc::all
)]
// https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
#![warn(
    absolute_paths_not_starting_with_crate,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    keyword_idents,
    let_underscore_drop,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    non_ascii_idents,
    noop_method_call,
    pointer_structural_match,
    rust_2021_incompatible_closure_captures,
    rust_2021_incompatible_or_patterns,
    rust_2021_prefixes_incompatible_syntax,
    rust_2021_prelude_collisions,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_code,
    unsafe_op_in_unsafe_fn,
    unstable_features,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_macro_rules,
    unused_qualifications,
    unused_results,
    unused_tuple_struct_fields,
    variant_size_differences
)]
#![allow(
    clippy::blanket_clippy_restriction_lints,
    clippy::cargo_common_metadata,
    clippy::expect_used,
    clippy::implicit_return,
    clippy::inline_always,
    clippy::let_underscore_untyped,
    clippy::min_ident_chars,
    clippy::missing_trait_methods,
    clippy::mod_module_files,
    clippy::multiple_unsafe_ops_per_block,
    clippy::needless_borrowed_reference,
    clippy::partial_pub_fields,
    clippy::pub_use,
    clippy::pub_with_shorthand,
    clippy::question_mark_used,
    clippy::redundant_pub_crate,
    clippy::ref_patterns,
    clippy::semicolon_outside_block,
    clippy::separated_literal_suffix,
    clippy::similar_names,
    clippy::single_call_fn,
    clippy::single_char_lifetime_names,
    clippy::std_instead_of_alloc,
    clippy::string_add,
    clippy::use_self,
    clippy::wildcard_imports
)]

/// Unwrap if we're debugging but `unwrap_unchecked` if we're not.
#[cfg(any(debug_assertions, test))]
macro_rules! unwrap {
    ($expr:expr) => {
        $expr.unwrap()
    };
}

/// Unwrap if we're debugging but `unwrap_unchecked` if we're not.
#[cfg(not(any(debug_assertions, test)))]
macro_rules! unwrap {
    ($expr:expr) => {{
        #[allow(unsafe_code)]
        let result = unsafe { $expr.unwrap_unchecked() };
        result
    }};
}

/// Unwrap if we're debugging but `unwrap_unchecked` if we're not.
#[cfg(any(debug_assertions, test))]
macro_rules! get {
    ($expr:expr, $index:expr) => {
        $expr.get($index).unwrap()
    };
}

/// Unwrap if we're debugging but `unwrap_unchecked` if we're not.
#[cfg(not(any(debug_assertions, test)))]
macro_rules! get {
    ($expr:expr, $index:expr) => {{
        #[allow(unsafe_code)]
        let result = unsafe { $expr.get_unchecked($index) };
        result
    }};
}

/// Unreachable state, but checked if we're debugging.
#[cfg(any(debug_assertions, test))]
macro_rules! never {
    () => {
        unreachable!()
    };
}

/// Unreachable state, but checked if we're debugging.
#[cfg(not(any(debug_assertions, test)))]
macro_rules! never {
    () => {{
        #[allow(unsafe_code)]
        unsafe {
            core::hint::unreachable_unchecked()
        }
    }};
}

/*
/// Unwrap if we're debugging but `unwrap_unchecked` if we're not.
#[cfg(any(debug_assertions, test))]
macro_rules! get_mut {
    ($expr:expr, $index:expr) => {
        $expr.get_mut($index).unwrap()
    };
}

/// Unwrap if we're debugging but `unwrap_unchecked` if we're not.
#[cfg(not(any(debug_assertions, test)))]
macro_rules! get_mut {
    ($expr:expr, $index:expr) => {{
        #[allow(unsafe_code, unused_unsafe)]
        let result = unsafe { $expr.get_unchecked_mut($index) };
        result
    }};
}
*/

/// Call a function that will also be available to the compiled parser.
#[macro_export]
macro_rules! call {
    ($ex:expr) => {
        $crate::Call::new($ex, stringify!($ex).to_owned())
    };
}

mod automaton;
mod call;
mod curry_opt;
mod edge;
mod exec;
mod indices;
mod lookup;
mod merge;
mod range;
mod run;
mod state;
mod subc;
mod wildcard;

#[cfg(feature = "quickcheck")]
mod quickcheck_impl;

#[cfg(test)]
mod test;

pub use {
    automaton::{Automaton, Deterministic, Nondeterministic},
    call::Call,
    curry_opt::CurryOpt,
    edge::Edge,
    exec::{Execute, Execution, IllFormed},
    indices::Indices,
    lookup::{Lookup, Return},
    merge::{merge, Merge},
    range::Range,
    run::Run,
    state::State,
    wildcard::Wildcard,
};

#[cfg(all(test, not(feature = "quickcheck")))]
use quickcheck as _;
