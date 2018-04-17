//! ```
//! lazy_extern! {
//!     libgroup SHELL_CORE: ShCoreItems;
//!     lib ShCore = "ShCore.dll";
//! 
//!     #[lib(ShCore)]
//!     #[feature_test(is_dpi_awareness_available)]
//!     /// Set the Dpi (v1) awareness without a manifest
//!     extern "stdcall" fn SetProcessDpiAwareness(value: u32) -> i32;
//! }
//! ```

pub extern crate libloading;

use std::ffi::OsStr;

#[doc(hidden)]
pub trait ExternFnPtr {
    type FnPtr;
}

#[doc(hidden)]
pub trait LoadFnPtr<G> {
    fn load_from_lib(libs: &mut G);
}

#[macro_export]
/// See the [module documentation](index.html)
macro_rules! lazy_extern {
    (
        libgroup $groupname:ident : $grouptype:ident;
        $(lib $libname:ident = $libpath:expr;)*
        $(
            $(#[$($meta:tt)*])*
            extern $abi:tt fn $fnname:ident($($argname:ident : $argty:ty),*) -> $retty:ty;
        )*
    ) => {
        $(
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            pub enum $fnname {}
            impl $crate::ExternFnPtr for $fnname {
                type FnPtr = unsafe extern $abi fn($($argname : $argty),*) -> $retty;
            }

            lazy_extern! {
                @process_fn_meta
                $groupname $grouptype
                $abi $fnname($($argname : $argty),*; $retty);
                $(#[$($meta)*])*
            }
        )*

        #[allow(non_snake_case)]
        pub struct $grouptype {
            pub $($libname : Option<$crate::libloading::Library>,)*
            pub $($fnname : Option<<$fnname as $crate::ExternFnPtr>::FnPtr>,)*
        }

        lazy_static! {
            pub static ref $groupname : $grouptype = {
                #[allow(non_snake_case)] {
                    $(let $libname = $crate::LibSpecifier::load_lib($libpath).ok();)*
                    $(let $fnname = None;)*
                    let mut group = $grouptype {
                        $($libname,)*
                        $($fnname,)*
                    };

                    $(<$fnname as $crate::LoadFnPtr<$grouptype>>::load_from_lib(&mut group);)*

                    group
                }
            };
        }
    };

    // No meta items left
    (
        @process_fn_meta
        $groupname:ident $grouptype:ident
        $abi:tt $fnname:ident($($argname:ident : $argty:ty),*; $retty:ty);
        $(; doc $doc:expr)*
    ) => {
        #[allow(non_snake_case)]
        $(#[doc = $doc])*
        pub unsafe fn $fnname($($argname : $argty),*) -> $retty {
            ($groupname.$fnname
                .expect("You should use feature_test if you don't know whether this function will be loaded")
            )($($argname),*)
        }
    };


    (
        @process_fn_meta
        $groupname:ident $grouptype:ident
        $abi:tt $fnname:ident($($argname:ident : $argty:ty),*; $retty:ty);
        #[doc = $meta:expr]
        $(#[$($rest:tt)*])*
        $(; doc $doc:expr)*
    ) => {
        lazy_extern! {
            @process_fn_meta
            $groupname $grouptype
            $abi $fnname($($argname : $argty),*; $retty);
            $(#[$($rest)*])*
            $(; doc $doc)*
            ; doc $meta
        }
    };

    (
        @process_fn_meta
        $groupname:ident $grouptype:ident
        $abi:tt $fnname:ident($($argname:ident : $argty:ty),*; $retty:ty);
        #[lib($libname:ident)]
        $(#[$($rest:tt)*])*
        $(; doc $doc:expr)*
    ) => {
        #[doc(hidden)]
        impl $crate::LoadFnPtr<$grouptype> for $fnname {
            fn load_from_lib(libs: &mut $grouptype) {
                let sym = ::std::option::Option::as_ref(&libs.$libname)
                    .and_then(|lib| {
                        unsafe {
                            let func: $crate::libloading::Symbol<<$fnname as $crate::ExternFnPtr>::FnPtr> =
                                lib.get(concat!(stringify!($fnname), "\0").as_bytes()).ok()?;
                            Some(*func)
                        }
                    });
                libs.$fnname = sym;
            }
        }

        lazy_extern! {
            @process_fn_meta
            $groupname $grouptype
            $abi $fnname($($argname : $argty),*; $retty);
            $(#[$($rest)*])*
            $(; doc $doc)*
        }
    };

    (
        @process_fn_meta
        $groupname:ident $grouptype:ident
        $abi:tt $fnname:ident($($argname:ident : $argty:ty),*; $retty:ty);
        #[feature_test($feature:ident)]
        $(#[$($rest:tt)*])*
        $(; doc $doc:expr)*
    ) => {
        pub fn $feature() -> bool {
            ::std::option::Option::is_some(&$groupname.$fnname)
        }

        lazy_extern! {
            @process_fn_meta
            $groupname $grouptype
            $abi $fnname($($argname : $argty),*; $retty);
            $(#[$($rest)*])*
            $(; doc $doc)*
        }
    };
}

/// Describes how to load a library from a given value for a `lib X = Expr;` statement
/// in the `lazy_extern!` declaration. For `&str` or `&OsStr` it calls `Library::new`
/// and passes the string as the library name to load. More complicated expressions are
/// also allowed with brackets, as long as it evaluates to a type implementing LibSpecifier.
pub trait LibSpecifier {
    /// Load the library
    fn load_lib(spec: Self) -> libloading::Result<libloading::Library>;
}

impl<'a> LibSpecifier for &'a OsStr {
    /// Load the library
    fn load_lib(spec: Self) -> libloading::Result<libloading::Library> {
        libloading::Library::new(spec)
    }
}

impl<'a> LibSpecifier for &'a str {
    /// Load the library
    fn load_lib(spec: Self) -> libloading::Result<libloading::Library> {
        libloading::Library::new(spec)
    }
}

/// Passthrough implementation
impl LibSpecifier for libloading::Result<libloading::Library> {
    fn load_lib(spec: Self) -> libloading::Result<libloading::Library> {
        spec
    }
}

/// Passthrough implementation
impl LibSpecifier for libloading::Library {
    fn load_lib(spec: Self) -> libloading::Result<libloading::Library> {
        Ok(spec)
    }
}
