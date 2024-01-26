#![allow(dead_code)]
#![feature(try_blocks)]
#![feature(impl_trait_in_assoc_type)]
#![feature(trait_alias)]

mod accn;
mod journal;
mod valuable;

mod repl;
#[cfg(test)]
mod tests;

fn main() {
    repl::repl();
}
