use std::pin::Pin;
use super::{IMMTLSPServer,ServerWrapper};
use std::{io::{self,ErrorKind},task::{Context,Poll}};
use async_lsp::{client_monitor::ClientProcessMonitorLayer, concurrency::ConcurrencyLayer, panic::CatchUnwindLayer, router::Router, server::LifecycleLayer, tracing::TracingLayer, ClientSocket, LspService, MainLoop};
use axum::extract::ws::Message;
use tower::ServiceBuilder;

pub fn upgrade<T:IMMTLSPServer+Send+'static>(ws:axum::extract::WebSocketUpgrade,new:impl FnOnce(ClientSocket) -> T + Send + 'static) -> axum::response::Response {
  ws.on_upgrade(|ws| {
    let (server,_) = async_lsp::MainLoop::new_server(|client| {
      //let server = ServerWrapper::new(new(client.clone()));
      ServiceBuilder::new()
      .layer(TracingLayer::default())
      .layer(LifecycleLayer::default())
      .layer(CatchUnwindLayer::default())
      .layer(ConcurrencyLayer::default())
      .layer(ClientProcessMonitorLayer::new(client.clone()))
      .service(ServerWrapper::new(new(client)).router())
    });
    let socket = SocketWrapper {
      inner:std::sync::Arc::new(parking_lot::Mutex::new(ws)),
      read_buf:Vec::new()
    };
    run(socket,server)
  })
}

#[allow(clippy::future_not_send)]
async fn run<T: LspService<Response = serde_json::value::Value>>(socket:SocketWrapper,main:MainLoop<T>)
where async_lsp::ResponseError: From<T::Error> {
  if let Err(e) = main.run_buffered(socket.clone(), socket).await {
    tracing::error!("Error: {:?}",e);
  }

}

#[derive(Clone)]
struct SocketWrapper {
  inner:std::sync::Arc<parking_lot::Mutex<axum::extract::ws::WebSocket>>,
  read_buf:Vec<u8>
}


impl SocketWrapper {
  fn poll_read_internal(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut [u8],
  ) -> Poll<io::Result<usize>> {
    let this = self.get_mut();
    let mut lock = this.inner.lock();
    let inner = Pin::new(&mut *lock);
    let r = futures::Stream::poll_next(inner, cx);
    drop(lock);

    r.map(|result| match result {
      None => Ok(0),
      Some(Err(e)) => Err(io::Error::new(ErrorKind::Other, e)),
      Some(Ok(message)) => match message {
        Message::Text(text) => Ok(Self::handle_incoming_data(buf, text.as_bytes(), &mut this.read_buf)),
        Message::Binary(binary) => Ok(Self::handle_incoming_data(buf, &binary, &mut this.read_buf)),
        Message::Close(_) => Err(io::Error::new(ErrorKind::BrokenPipe, "WebSocket closed")),
        Message::Ping(_) | Message::Pong(_) => Ok(0), // Ignore control frames
      },
    })
  }

  fn handle_incoming_data(
    buf: &mut [u8],
    data: &[u8],
    read_buf: &mut Vec<u8>,
  ) -> usize {
    let data_len = data.len();
    let buf_len = buf.len();
    if data_len > buf_len {
      buf.copy_from_slice(&data[..buf_len]);
      read_buf.extend_from_slice(&data[buf_len..]);
      buf_len
    } else {
      buf[..data_len].copy_from_slice(data);
      data_len
    }
  }
}


impl futures::AsyncRead for SocketWrapper {
  fn poll_read(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut [u8],
  ) -> Poll<io::Result<usize>> {
    if !self.read_buf.is_empty() {
      let to_copy = std::cmp::min(buf.len(), self.read_buf.len());
      buf[..to_copy].copy_from_slice(&self.read_buf[..to_copy]);
      self.read_buf.drain(..to_copy);
      return Poll::Ready(Ok(to_copy));
    }
    self.as_mut().poll_read_internal(cx, buf)
  }
}

impl futures::AsyncBufRead for SocketWrapper {
  fn poll_fill_buf(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<io::Result<&[u8]>> {
    if self.read_buf.is_empty() {
      match self.as_mut().poll_read_internal(cx, &mut []) {
        Poll::Ready(Ok(0)) | Poll::Pending => (),
        Poll::Ready(Ok(_)) => unreachable!(),
        Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
      }
    }

    let this = self.into_ref().get_ref();
    if this.read_buf.is_empty() {
      Poll::Pending
    } else {
      Poll::Ready(Ok(&this.read_buf))
    }
  }

  fn consume(self: Pin<&mut Self>, amt: usize) {
    let this = self.get_mut();
    this.read_buf.drain(..std::cmp::min(amt, this.read_buf.len()));
  }
}

impl futures::AsyncWrite for SocketWrapper {
  fn poll_write(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &[u8],
  ) -> Poll<io::Result<usize>> {
    let message = Message::Text(std::string::String::from_utf8_lossy(buf).to_string());
    let mut lock = self.inner.lock();
    let inner = Pin::new(&mut *lock);

    match futures::Sink::poll_ready(inner, cx) {
      Poll::Pending => Poll::Pending,
      Poll::Ready(Err(e)) => Poll::Ready(Err(io::Error::new(ErrorKind::Other, e))),
      Poll::Ready(Ok(())) => Poll::Ready(futures::Sink::start_send(Pin::new(&mut *lock), message)
        .map(|()| buf.len())
        .map_err(|e| io::Error::new(ErrorKind::Other, e)))
    }
  }

  #[allow(clippy::significant_drop_tightening)]
  fn poll_flush(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<io::Result<()>> {
    let mut lock = self.inner.lock();
    let inner = Pin::new(&mut *lock);
    futures::Sink::poll_flush(inner, cx)
      .map_err(|e| io::Error::new(ErrorKind::Other, e))
  }

  #[allow(clippy::significant_drop_tightening)]
  fn poll_close(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<io::Result<()>> {
    let mut lock = self.inner.lock();
    let inner = Pin::new(&mut *lock);
    futures::Sink::poll_close(inner, cx)
      .map_err(|e| io::Error::new(ErrorKind::Other, e))
  }
}
