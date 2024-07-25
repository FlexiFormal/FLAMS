use immt_core::building::formats::BuildTargetId;
use immt_core::short_id;
use crate::building::targets::SourceFormat;
use crate::building::tasks::BuildTask;
use crate::controller::{Controller};

short_id!(?ExtensionId);

pub trait MMTExtension:Send+Sync+std::fmt::Debug {
    fn name(&self) -> ExtensionId;
    fn on_plugin_load(&self,_controller:&dyn Controller) {}
    fn as_formats(&self) -> Option<&dyn FormatExtension> { None }
}

#[async_trait::async_trait]
pub trait FormatExtension:MMTExtension {
    fn formats(&self) -> Vec<SourceFormat>;
    fn sandbox(&self, controller:&mut dyn Controller) -> Box<dyn MMTExtension>;
    fn get_deps(&self,controller:&dyn Controller, task:&BuildTask);
    fn build(&self,ctrl:&dyn Controller,task:&BuildTask,target:BuildTargetId,index:u8) -> bool;
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
