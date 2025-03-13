use futures::{
    future::BoxFuture,
    task::{waker_ref, ArcWake, UnsafeFutureObj},
    Future, FutureExt,
};
use once_cell::sync::Lazy;
use std::{
    pin::Pin,
    sync::{
        self,
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
    task::{Context, Poll},
};
use tokio::sync::oneshot;

pub static RUNTIME: Lazy<Runtime> = Lazy::new(Runtime::new);

pub struct Runtime {
    sender: Sender<Arc<Task>>,
    receiver: Mutex<Receiver<Arc<Task>>>,
}

impl Runtime {
    fn new() -> Self {
        let (sender, receiver) = sync::mpsc::channel();
        let receiver = Mutex::new(receiver);
        Self { sender, receiver }
    }
}

pub fn async_spawn<T: Send + 'static>(
    future: impl Future<Output = T> + 'static + Send,
) -> JoinHandler<T> {
    let (sender, receiver) = oneshot::channel();
    let future = async move {
        drop(sender.send(future.await));
    };
    RUNTIME
        .sender
        .send(Arc::new(Task {
            future: Mutex::new(Some(future.boxed())),
        }))
        .unwrap();
    JoinHandler::new(receiver)
}

pub fn async_block_on<'a, T: Send + 'static, F>(future: F) -> T
where
    F: Future<Output = T> + 'a + Send,
{
    let (sender, receiver) = sync::mpsc::channel();
    let future = unsafe {
        let future = std::mem::transmute::<
            *mut (dyn Future<Output = T> + '_),
            *mut (dyn Future<Output = T> + Send + 'static),
        >(Box::new(future).into_raw());
        Pin::new_unchecked(Box::from_raw(future))
    };
    let future = async move {
        drop(sender.send(future.await));
    };
    let task = Task {
        future: Mutex::new(Some(future.boxed())),
    };
    RUNTIME.sender.send(Arc::new(task)).unwrap();
    receiver.recv().unwrap()
}

pub fn async_worker_run() {
    let task = {
        if let Ok(lock) = RUNTIME.receiver.try_lock() {
            lock.try_recv()
        } else {
            return;
        }
    };
    match task {
        Ok(task) => {
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&waker);
                if future.as_mut().poll(context).is_pending() {
                    *future_slot = Some(future);
                }
            }
        }
        Err(sync::mpsc::TryRecvError::Disconnected) => panic!(),
        _ => (),
    }
}

pub struct Task {
    pub future: Mutex<Option<BoxFuture<'static, ()>>>,
}

unsafe impl Sync for Task {}

unsafe impl Send for Task {}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let cloned = arc_self.clone();
        RUNTIME.sender.send(cloned).unwrap()
    }
}

pub struct JoinHandler<T: Send + 'static> {
    receiver: oneshot::Receiver<T>,
}

impl<T: Send + 'static> JoinHandler<T> {
    fn new(receiver: oneshot::Receiver<T>) -> Self {
        Self { receiver }
    }
}

impl<T: Send + 'static> Future for JoinHandler<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.receiver).poll(cx).map(|x| x.unwrap())
    }
}
