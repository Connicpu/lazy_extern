#[macro_use]
extern crate lazy_extern;
#[macro_use]
extern crate lazy_static;

lazy_extern! {
    libgroup SHELL_CORE: ShCoreItems;
    lib ShCore = "ShCore.dll";

    #[lib(ShCore)]
    #[feature_test(is_dpi_awareness_available)]
    /// Set the Dpi (v1) awareness without a manifest
    extern "stdcall" fn SetProcessDpiAwareness(value: u32) -> i32;
}

fn main() {
    if is_dpi_awareness_available() {
        unsafe { SetProcessDpiAwareness(2); }
    }
}
