use std::sync::Arc;

use smol::{
    channel::{Receiver, Sender},
    lock::Mutex,
};

use crate::runtime;

pub struct ServerSentEvents<E>
where
    E: Sync + Send + 'static,
{
    channel: Receiver<Arc<E>>, // subscribers: Arc<Mutex<Subscribers<E>>>,
}

impl<E> ServerSentEvents<E>
where
    E: Sync + Send + 'static,
{
    const fn new(channel: Receiver<Arc<E>>) -> Self {
        Self { channel }
    }
}

impl<E> AsyncIterator for ServerSentEvents<E>
where
    E: Sync + Send + 'static,
{
    type Item = Arc<E>;

    async fn next(&mut self) -> Option<Self::Item> {
        self.channel.recv().await.ok()
    }
}

impl<T: Iterator> AsyncIterator for T {
    type Item = T::Item;

    fn next(&mut self) -> impl Future<Output = Option<Self::Item>> {
        async { self.next() }
    }
}

/// Naive implementation for a thread safe, multi producer event queue for server sent events
pub struct EventManager<E>
where
    E: Send + Sync + 'static,
{
    sender: Sender<E>,
    subscribers: Arc<Mutex<Vec<Sender<Arc<E>>>>>,
}

pub trait AsyncIterator {
    type Item;
    fn next(&mut self) -> impl Future<Output = Option<Self::Item>>;
}

impl<E: Sync + Send + 'static> EventManager<E> {
    pub fn new() -> Self {
        let (sender, receiver) = smol::channel::unbounded::<E>();

        let manager = EventManager {
            sender,
            subscribers: Default::default(),
        };

        let subscribers = manager.subscribers.clone();

        let task = async move {
            while let Ok(msg) = receiver.recv().await.map(Arc::new) {
                let mut lock = subscribers.lock().await;
                lock.retain(|subscriber| subscriber.send_blocking(msg.clone()).is_ok());
            }
        };
        runtime::task(task);

        manager
    }

    pub fn sender(&self) -> Sender<E> {
        self.sender.clone()
    }

    pub async fn consumer(&self) -> ServerSentEvents<E> {
        let (sender, receiver) = smol::channel::unbounded::<Arc<E>>();
        self.subscribers.lock().await.push(sender);
        ServerSentEvents::new(receiver)
    }
}

pub trait EventSource {
    type Event: Send + Sync + 'static;
}
