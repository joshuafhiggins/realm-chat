use godot::prelude::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}
