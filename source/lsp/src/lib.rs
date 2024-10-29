mod implementation;
#[cfg(feature="ws")]
pub mod ws;

use async_lsp::ClientSocket;
pub use async_lsp;

pub trait IMMTLSPServer {
  fn client(&mut self) -> &mut ClientSocket;
  fn initialized(&mut self) {}
}


pub struct ServerWrapper<T:IMMTLSPServer> {
  pub inner:T
}
impl <T:IMMTLSPServer> ServerWrapper<T> {
  #[inline]
  pub const fn new(inner:T) -> Self {
    Self { inner }
  }
}