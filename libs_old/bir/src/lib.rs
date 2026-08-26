/// Intermediate representation.
/// Responsible for filling out intrinsics, resolve impl blocks / `fn on int add`
/// resolving syntax sugar like += and so on.
/// Mangling names.
/// Resolving named function parameters.
/// This IR has pointers.
/// First thing will prob be to make array push an intrinsic
pub mod ast;
mod bir;
pub use bir::bir;
