use std::any::Any;
use immt_core::building::formats::ShortId;
use crate::building::targets::SourceFormat;
use crate::controller::Controller;

pub trait MMTExtension:Send+Sync {
    fn name(&self) -> ShortId;
    fn on_plugin_load(&self,_controller:&dyn Controller) {}
    
    fn test(&self, controller:&mut dyn Controller) -> bool;
    fn test2(&self, controller:&mut dyn Controller) -> bool;
    fn as_formats(&self) -> Option<&dyn FormatExtension> { None }
}

pub trait FormatExtension:MMTExtension {
    fn formats(&self) -> Vec<SourceFormat>;
    fn sandbox(&self, controller:&mut dyn Controller) -> Box<dyn MMTExtension>;
}

#[derive(Copy, Clone)]
pub struct ExtensionDeclaration {
    pub rustc_version: &'static str,
    pub version: &'static str,
    pub register: unsafe extern "C" fn() -> Box<dyn MMTExtension>,
    pub dependencies: &'static [(&'static str,&'static str)]
}

#[macro_export]
macro_rules! export_plugin {
    ($register:expr,$(($name:literal,$version:literal)),+) => {
        #[doc(hidden)]
        #[no_mangle]
        pub static extension_declaration: $crate::extensions::ExtensionDeclaration = {
            $crate::extensions::ExtensionDeclaration {
                rustc_version: $crate::RUSTC_VERSION,
                version: $crate::API_VERSION,
                register:$register,
                dependencies: &[$(($name,$version)),+]
            }
        };
    };
    ($register:expr) => {
        #[doc(hidden)]
        #[no_mangle]
        pub static extension_declaration: $crate::extensions::ExtensionDeclaration = {
            $crate::extensions::ExtensionDeclaration {
                rustc_version: $crate::RUSTC_VERSION,
                version: $crate::API_VERSION,
                register:$register,
                dependencies: &[]
            }
        };
    };
}
