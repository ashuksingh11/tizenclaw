//! Event bus — publish/subscribe system for internal events.

use serde_json::Value;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Condvar, Mutex};

const MAX_QUEUE_SIZE: usize = 1000;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EventType {
    AppInstalled,
    AppUninstalled,
    AppLaunched,
    AppTerminated,
    BatteryChanged,
    NetworkChanged,
    ScreenStateChanged,
    SystemEvent,
    Custom(String),
}

#[derive(Clone, Debug)]
pub struct SystemEvent {
    pub event_type: EventType,
    pub source: String,
    pub data: Value,
    pub timestamp: u64,
}

impl Default for SystemEvent {
    fn default() -> Self {
        SystemEvent {
            event_type: EventType::SystemEvent,
            source: String::new(),
            data: Value::Null,
            timestamp: 0,
        }
    }
}

type EventCallback = Box<dyn Fn(&SystemEvent) + Send + Sync>;

struct Subscription {
    id: i32,
    event_type: EventType,
    match_all: bool,
    callback: EventCallback,
}

pub struct EventBus {
    running: Arc<AtomicBool>,
    queue: Arc<(Mutex<VecDeque<SystemEvent>>, Condvar)>,
    subscribers: Arc<Mutex<Vec<Subscription>>>,
    next_id: AtomicI32,
}

impl EventBus {
    pub fn new() -> Self {
        EventBus {
            running: Arc::new(AtomicBool::new(false)),
            queue: Arc::new((Mutex::new(VecDeque::new()), Condvar::new())),
            subscribers: Arc::new(Mutex::new(vec![])),
            next_id: AtomicI32::new(1),
        }
    }

    pub fn start(&self) -> Option<std::thread::JoinHandle<()>> {
        if self.running.load(Ordering::SeqCst) {
            return None;
        }
        self.running.store(true, Ordering::SeqCst);

        let running = self.running.clone();
        let queue = self.queue.clone();
        let subscribers = self.subscribers.clone();

        let handle = std::thread::spawn(move || {
            Self::dispatch_loop(running, queue, subscribers);
        });

        log::info!("EventBus started");
        Some(handle)
    }

    pub fn stop(&self) {
        if !self.running.load(Ordering::SeqCst) {
            return;
        }
        self.running.store(false, Ordering::SeqCst);
        self.queue.1.notify_all();
        log::info!("EventBus stopped");
    }

    pub fn publish(&self, mut event: SystemEvent) {
        if event.timestamp == 0 {
            event.timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
        }

        let (lock, cvar) = &*self.queue;
        if let Ok(mut q) = lock.lock() {
            if q.len() >= MAX_QUEUE_SIZE {
                q.pop_front();
            }
            q.push_back(event);
        }
        cvar.notify_one();
    }

    pub fn subscribe(&self, event_type: EventType, callback: impl Fn(&SystemEvent) + Send + Sync + 'static) -> i32 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        if let Ok(mut subs) = self.subscribers.lock() {
            subs.push(Subscription {
                id,
                event_type,
                match_all: false,
                callback: Box::new(callback),
            });
        }
        id
    }

    pub fn subscribe_all(&self, callback: impl Fn(&SystemEvent) + Send + Sync + 'static) -> i32 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        if let Ok(mut subs) = self.subscribers.lock() {
            subs.push(Subscription {
                id,
                event_type: EventType::SystemEvent,
                match_all: true,
                callback: Box::new(callback),
            });
        }
        id
    }

    pub fn unsubscribe(&self, subscription_id: i32) {
        if let Ok(mut subs) = self.subscribers.lock() {
            subs.retain(|s| s.id != subscription_id);
        }
    }

    fn dispatch_loop(
        running: Arc<AtomicBool>,
        queue: Arc<(Mutex<VecDeque<SystemEvent>>, Condvar)>,
        subscribers: Arc<Mutex<Vec<Subscription>>>,
    ) {
        let (lock, cvar) = &*queue;
        while running.load(Ordering::SeqCst) {
            let event = {
                let mut q = lock.lock().unwrap();
                while q.is_empty() && running.load(Ordering::SeqCst) {
                    q = cvar.wait(q).unwrap();
                }
                if !running.load(Ordering::SeqCst) && q.is_empty() {
                    break;
                }
                q.pop_front()
            };

            if let Some(event) = event {
                if let Ok(subs) = subscribers.lock() {
                    for sub in subs.iter() {
                        if sub.match_all || sub.event_type == event.event_type {
                            (sub.callback)(&event);
                        }
                    }
                }
            }
        }
    }
}
