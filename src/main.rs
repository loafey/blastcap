#![feature(impl_trait_in_bindings, never_type)]
mod args;
mod game;
mod gui;
mod network;

fn main() {
    gui::start();
}
