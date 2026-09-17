use bootloader_api::info::{MemoryRegion, MemoryRegionKind, MemoryRegions};
use x86_64::{PhysAddr, VirtAddr, structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB, page}};

pub struct BootInfoFrameAllocator {
    mem_regs: &'static MemoryRegions,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(mem_regs: &'static MemoryRegions) -> Self {
        BootInfoFrameAllocator { 
            mem_regs, 
            next: 0 
        }
    }

    fn get_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.mem_regs.iter();
        let usable_regions = regions.filter(|r| r.kind == MemoryRegionKind::Usable);
        let addr_range = usable_regions.map(|r| r.start..r.end);
        let frame_addresses = addr_range.flat_map(|r| r.step_by(4096)); // Step by 4KiB as thats the Frame Size
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<x86_64::structures::paging::PhysFrame<Size4KiB>> {
        let frame = self.get_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

pub unsafe fn init(phys_mem_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = unsafe { enable_lvl4_table(phys_mem_offset) };
    unsafe { OffsetPageTable::new(level_4_table, phys_mem_offset) }
}

unsafe fn enable_lvl4_table(phys_mem_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (lvl_4_table_frame, _) = Cr3::read();

    let phys = lvl_4_table_frame.start_address();
    let virt = phys_mem_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}