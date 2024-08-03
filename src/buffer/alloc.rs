use std::collections::HashMap;

pub trait BufferAlloc<T> {
    fn get(&self, id: &str) -> Option<&BufferAllocation>;
    fn size(&self) -> usize;
}

pub trait BufferDynamicAlloc<T>: BufferAlloc<T> {
    fn allocate(&mut self, id: &str, size: usize) -> usize;
    fn free(&mut self, id: &str) -> Option<BufferAllocation>;
}

#[derive(Debug, Default)]
pub struct BufferDynamicAllocator {
    packets: HashMap<BufferAllocationID, BufferAllocation>,
    pub size: usize,
}

impl<T: bytemuck::Pod + bytemuck::Zeroable> BufferAlloc<T> for BufferDynamicAllocator {
    fn get(&self, id: &str) -> Option<&BufferAllocation> {
        self.packets.get(id)
    }

    fn size(&self) -> usize {
        self.size
    }
}

impl<T: bytemuck::Pod + bytemuck::Zeroable> BufferDynamicAlloc<T> for BufferDynamicAllocator {
    fn allocate(&mut self, id: &str, size: usize) -> usize {
        let offset = self.size;
        self.size += size;
        self.packets
            .insert(id.to_string(), BufferAllocation { offset, size });

        offset
    }

    fn free(&mut self, id: &str) -> Option<BufferAllocation> {
        if let Some(remove_packet) = self.packets.remove(id) {
            self.size -= remove_packet.size;

            // Update offsets of all packets after the removed one
            for packet in self.packets.values_mut() {
                if packet.offset > remove_packet.offset {
                    packet.offset -= remove_packet.size;
                }
            }

            Some(remove_packet)
        } else {
            None
        }
    }
}

pub type BufferAllocationID = String;

#[derive(Debug)]
pub struct BufferAllocation {
    pub offset: usize,
    pub size: usize,
}
