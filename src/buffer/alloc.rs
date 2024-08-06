use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicUsize},
        mpsc::{SendError, Sender},
        Arc,
    },
};

type FnModifyData<T> = Box<dyn FnMut(&mut [T])>;

pub struct ModifyAction<T> {
    pub start: usize,
    pub end: usize,
    mod_action: FnModifyData<T>,
}

impl<T> ModifyAction<T> {
    pub(super) fn act(&mut self, data: &mut [T]) {
        (self.mod_action)(&mut data[self.start..=self.end]);
    }
}

pub trait AllocHandle<T> {
    fn offset(&self) -> usize;
    fn size(&self) -> usize;

    fn get_action_sender(&self) -> &Sender<ModifyAction<T>>;

    fn send_action(&self, mut action: ModifyAction<T>) -> Result<(), SendError<ModifyAction<T>>> {
        action.start += self.offset();
        action.end += self.offset();

        self.get_action_sender().send(action)
    }
}

pub struct StaticAllocHandle<T> {
    pub id: String,
    pub offset: usize,
    pub size: usize,

    action_sender: Sender<ModifyAction<T>>,
}

impl<T> std::hash::Hash for StaticAllocHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> AllocHandle<T> for StaticAllocHandle<T> {
    fn offset(&self) -> usize {
        self.offset
    }

    fn size(&self) -> usize {
        self.size
    }

    fn get_action_sender(&self) -> &Sender<ModifyAction<T>> {
        &self.action_sender
    }
}

#[derive(Debug)]
pub struct DynamicAllocHandle<T> {
    id: String,
    destroyed: AtomicBool,
    destroy_sender: std::sync::mpsc::Sender<BufferAllocationID>,
    offset: AtomicUsize,
    size: AtomicUsize,

    action_sender: Sender<ModifyAction<T>>,
}

impl<T> std::hash::Hash for DynamicAllocHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> AllocHandle<T> for DynamicAllocHandle<T> {
    fn offset(&self) -> usize {
        self.offset.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn size(&self) -> usize {
        self.size.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn get_action_sender(&self) -> &Sender<ModifyAction<T>> {
        &self.action_sender
    }
}

impl<T> DynamicAllocHandle<T> {
    fn new(
        id: String,
        allocation: BufferAllocation,
        destroy_sender: Sender<BufferAllocationID>,
        action_sender: Sender<ModifyAction<T>>,
    ) -> Self {
        Self {
            id,
            destroyed: AtomicBool::new(false),
            destroy_sender,
            offset: AtomicUsize::new(allocation.offset),
            size: AtomicUsize::new(allocation.size),

            action_sender,
        }
    }

    pub fn destroy(&self) {
        self.destroyed
            .store(true, std::sync::atomic::Ordering::Relaxed);

        self.destroy_sender
            .send(self.id.clone())
            .expect("Failed to send destroy request");
    }

    pub fn is_destroyed(&self) -> bool {
        self.destroyed.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn allocation(&self) -> BufferAllocation {
        BufferAllocation {
            offset: self.offset(),
            size: self.size(),
        }
    }

    fn move_offset_left(&self, pos: usize) {
        self.offset
            .fetch_sub(pos, std::sync::atomic::Ordering::Relaxed);
    }
}

pub trait BufferAlloc<T> {
    type Handle: AllocHandle<T>;

    fn get(&self, id: &str) -> Option<&Arc<Self::Handle>>;
    fn size(&self) -> usize;
    fn update(&self, modify: impl Fn(ModifyAction<T>));
}

pub trait BufferDynamicAlloc<T>: BufferAlloc<T, Handle = DynamicAllocHandle<T>> {
    fn allocate(&mut self, id: &str, size: usize) -> Arc<DynamicAllocHandle<T>>;
    fn free(&mut self, id: &str) -> Option<BufferAllocation>;
    fn check_destroyed(&mut self);
}

#[derive(Debug)]
pub struct BufferDynamicAllocator<T> {
    packets: HashMap<BufferAllocationID, Arc<DynamicAllocHandle<T>>>,

    destroy_requests: std::sync::mpsc::Receiver<BufferAllocationID>,
    dummy_destroy_sender: std::sync::mpsc::Sender<BufferAllocationID>,

    action_queue: std::sync::mpsc::Receiver<ModifyAction<T>>,
    dummy_action_sender: std::sync::mpsc::Sender<ModifyAction<T>>,

    pub size: usize,
}

impl<T> Default for BufferDynamicAllocator<T> {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        let (action_tx, action_rx) = std::sync::mpsc::channel();

        Self {
            packets: Default::default(),
            destroy_requests: rx,
            dummy_destroy_sender: tx,

            action_queue: action_rx,
            dummy_action_sender: action_tx,
            size: Default::default(),
        }
    }
}

impl<T> BufferAlloc<T> for BufferDynamicAllocator<T> {
    type Handle = DynamicAllocHandle<T>;

    fn get(&self, id: &str) -> Option<&Arc<DynamicAllocHandle<T>>> {
        self.packets.get(id)
    }

    fn size(&self) -> usize {
        self.size
    }

    fn update(&self, modify: impl Fn(ModifyAction<T>)) {
        while let Ok(action) = self.action_queue.try_recv() {
            modify(action);
        }
    }
}

impl<T> BufferDynamicAlloc<T> for BufferDynamicAllocator<T> {
    fn allocate(&mut self, id: &str, size: usize) -> Arc<DynamicAllocHandle<T>> {
        let offset = self.size;
        self.size += size;

        let handle = Arc::new(DynamicAllocHandle::new(
            id.to_string(),
            BufferAllocation { offset, size },
            self.dummy_destroy_sender.clone(),
            self.dummy_action_sender.clone(),
        ));

        self.packets.insert(id.to_string(), handle.clone());

        handle
    }

    fn free(&mut self, id: &str) -> Option<BufferAllocation> {
        if let Some(remove_packet) = self.packets.remove(id) {
            self.size -= remove_packet.size();

            // Update offsets of all packets after the removed one
            for packet in self.packets.values_mut() {
                if packet.offset() > remove_packet.offset() {
                    packet.move_offset_left(remove_packet.size());
                }
            }

            remove_packet.destroy();

            Some(remove_packet.allocation())
        } else {
            None
        }
    }

    fn check_destroyed(&mut self) {
        while let Ok(id) = self.destroy_requests.try_recv() {
            self.free(&id);
        }
    }
}

pub type BufferAllocationID = String;

#[derive(Debug, Clone)]
pub struct BufferAllocation {
    pub offset: usize,
    pub size: usize,
}
