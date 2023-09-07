use std::collections::hash_map::Keys;
use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::spawn;
use tokio::sync::oneshot::Receiver;

use crate::service::Service;
use crate::transport::{Request, Response};

pub trait ServiceHandle<State>: Send + Sync {
    fn serve<'a>(
        &'a self,
        state: Arc<State>,
        request: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = Vec<u8>> + Send + 'a>>;
}

impl<State, Req, Res, F> ServiceHandle<State> for fn(Arc<State>, Req) -> F
where
    State: Send + Sync,
    Req: DeserializeOwned + Send,
    Res: Serialize + Send,
    F: Future<Output = Res> + Send,
{
    fn serve<'a>(
        &'a self,
        state: Arc<State>,
        request: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = Vec<u8>> + Send + 'a>> {
        Box::pin(async move {
            let req: Req = serde_json::from_slice(&request).unwrap();
            let res = self(state, req).await;
            serde_json::to_vec(&res).unwrap()
        })
    }
}

pub struct App<State>
where
    State: Send + Sync + 'static,
{
    state: Arc<State>,
    services: HashMap<String, Pin<Box<dyn ServiceHandle<State>>>>,
}

impl<State> Default for App<State>
where
    State: Default + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<State> App<State>
where
    State: Send + Sync + 'static,
{
    pub fn new(state: Arc<State>) -> Self {
        Self {
            state,
            services: HashMap::new(),
        }
    }

    pub fn service<Req, Res, F>(&mut self, handler: Service<Arc<State>, Req, Res, F>) -> &mut Self
    where
        Req: DeserializeOwned + Send + 'static,
        Res: Serialize + Send,
        F: Future<Output = Res> + Send + 'static,
    {
        self.services.insert(handler.name, handler.handle_fn);
        self
    }

    /// Returns an iterator over the services in this app.
    pub fn services(&self) -> Keys<'_, String, Pin<Box<dyn ServiceHandle<State>>>> {
        self.services.keys()
    }

    /// App starts to listen on the given address
    ///
    /// This function will not return until the server is stops.
    pub async fn serve(self: Arc<Self>, listen_addr: SocketAddr, stop: Receiver<()>) {
        let app = self.clone();
        let handle = async move {
            let listener = TcpListener::bind(listen_addr).await.unwrap();
            while let Ok((stream, _)) = listener.accept().await {
                let _handle = spawn(app.clone().dispatch(stream));
            }
        };

        tokio::select! {
            _ = stop => {},
            _ = tokio::signal::ctrl_c() => {},
            _ = handle => {},
        }
    }

    /// The dispatch function is called for every incoming request.
    async fn dispatch(self: Arc<Self>, mut stream: TcpStream) {
        let (read, mut write) = stream.split();
        let mut buf_reader = BufReader::new(read);
        let mut buf = [0u8; 1024];
        let bytes_read = buf_reader.read(&mut buf[..]).await.unwrap();
        log::debug!("Received {} bytes", bytes_read);
        let request: Request = serde_json::from_slice(&buf[..bytes_read]).unwrap();
        log::info!("Request: {}", request.name);
        let service = self.services.get(&request.name).unwrap();
        let payload = service.serve(self.state.clone(), request.payload).await;
        let response = Response { payload };
        write
            .write(serde_json::to_vec(&response).unwrap().as_slice())
            .await
            .unwrap();
    }

    pub async fn stop(&self) {
        todo!()
    }
}
