use ash::vk;
use ash::vk::Flags;
use ash::vk::Handle;
use ash::vk::ObjectType;
use std::fmt;
use std::os::raw::{c_char, c_void};

ash::define_handle!(Allocator, UNKNOWN);
ash::define_handle!(Allocation, UNKNOWN);
ash::define_handle!(Pool, UNKNOWN);

#[allow(non_camel_case_types)]
pub type PFN_AllocateDeviceMemoryFunction = unsafe extern "system" fn(
    allocator: Allocator,
    memory_type: u32,
    memory: vk::DeviceMemory,
    size: vk::DeviceSize,
);

#[allow(non_camel_case_types)]
pub type PFN_FreeDeviceMemoryFunction = unsafe extern "system" fn(
    allocator: Allocator,
    memory_type: u32,
    memory: vk::DeviceMemory,
    size: vk::DeviceSize,
);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AllocatorCreateFlags(pub(crate) vk::Flags);
ash::vk_bitflags_wrapped!(AllocatorCreateFlags, 0b11, Flags);
impl AllocatorCreateFlags {
    pub const EXTERNALLY_SYNCHRONIZED: Self = AllocatorCreateFlags(0b1);
    pub const KHR_DEDICATED_ALLOCATION: Self = AllocatorCreateFlags(0b10);
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AllocationCreateFlags(pub(crate) vk::Flags);
ash::vk_bitflags_wrapped!(AllocationCreateFlags, 0b1111111, Flags);
impl AllocationCreateFlags {
    pub const DEDICATED_MEMORY: Self = AllocationCreateFlags(0b1);
    pub const NEVER_ALLOCATE: Self = AllocationCreateFlags(0b10);
    pub const MAPPED: Self = AllocationCreateFlags(0b100);
    pub const CAN_BECOME_LOST: Self = AllocationCreateFlags(0b1000);
    pub const CAN_MAKE_OTHER_LOST: Self = AllocationCreateFlags(0b10000);
    pub const USER_DATA_COPY_STRING: Self = AllocationCreateFlags(0b100000);
    pub const UPPER_ADDRESS: Self = AllocationCreateFlags(0b1000000);
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PoolCreateFlags(pub(crate) vk::Flags);
ash::vk_bitflags_wrapped!(PoolCreateFlags, 0b110, Flags);
impl PoolCreateFlags {
    pub const IGNORE_BUFFER_IMAGE_GRANULARITY: Self = PoolCreateFlags(0b10);
    pub const LINEAR_ALGORITHM: Self = PoolCreateFlags(0b100);
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecordFlags(pub(crate) vk::Flags);
ash::vk_bitflags_wrapped!(RecordFlags, 0b1, Flags);
impl RecordFlags {
    pub const FLUSH_AFTER_CALL: Self = RecordFlags(0b1);
}

#[repr(C)]
pub struct DeviceMemoryCallbacks {
    pub allocate: PFN_AllocateDeviceMemoryFunction,
    pub free: PFN_FreeDeviceMemoryFunction,
}

#[repr(C)]
pub struct VulkanFunctions {
    pub vk_get_physical_device_properties: vk::PFN_vkGetPhysicalDeviceProperties,
    pub vk_get_physical_device_memory_properties: vk::PFN_vkGetPhysicalDeviceMemoryProperties,
    pub vk_allocate_memory: vk::PFN_vkAllocateMemory,
    pub vk_free_memory: vk::PFN_vkFreeMemory,
    pub vk_map_memory: vk::PFN_vkMapMemory,
    pub vk_unmap_memory: vk::PFN_vkUnmapMemory,
    pub vk_flush_mapped_memory_ranges: vk::PFN_vkFlushMappedMemoryRanges,
    pub vk_invalidate_mapped_memory_ranges: vk::PFN_vkInvalidateMappedMemoryRanges,
    pub vk_bind_buffer_memory: vk::PFN_vkBindBufferMemory,
    pub vk_bind_image_memory: vk::PFN_vkBindImageMemory,
    pub vk_get_buffer_memory_requirements: vk::PFN_vkGetBufferMemoryRequirements,
    pub vk_get_image_memory_requirements: vk::PFN_vkGetImageMemoryRequirements,
    pub vk_create_buffer: vk::PFN_vkCreateBuffer,
    pub vk_destroy_buffer: vk::PFN_vkDestroyBuffer,
    pub vk_create_image: vk::PFN_vkCreateImage,
    pub vk_destroy_image: vk::PFN_vkDestroyImage,
}

#[repr(C)]
pub struct RecordSettings {
    pub flags: RecordFlags,
    pub file_path: *const c_char,
}

#[repr(C)]
pub struct AllocatorCreateInfo {
    pub flags: AllocatorCreateFlags,
    pub physical_device: vk::PhysicalDevice,
    pub device: vk::Device,
    pub preferred_large_heap_block_size: vk::DeviceSize,
    pub allocation_callbacks: *const vk::AllocationCallbacks,
    pub device_memory_callbacks: *const DeviceMemoryCallbacks,
    pub frame_in_use_count: u32,
    pub heap_size_limit: *const vk::DeviceSize,
    pub vulkan_functions: *const VulkanFunctions,
    pub record_settings: *const RecordSettings,
}

#[repr(C)]
pub struct StatInfo {
    pub block_count: u32,
    pub allocation_count: u32,
    pub unused_range_count: u32,
    pub used_bytes: vk::DeviceSize,
    pub unused_bytes: vk::DeviceSize,
    pub allocation_size_min: vk::DeviceSize,
    pub allocation_size_avg: vk::DeviceSize,
    pub allocation_size_max: vk::DeviceSize,
    pub unused_range_size_min: vk::DeviceSize,
    pub unused_range_size_avg: vk::DeviceSize,
    pub unused_range_size_max: vk::DeviceSize,
}

#[repr(C)]
pub struct Stats {
    pub memory_type: [StatInfo; vk::MAX_MEMORY_TYPES],
    pub memory_heap: [StatInfo; vk::MAX_MEMORY_HEAPS],
    pub total: StatInfo,
}

#[repr(C)]
pub enum MemoryUsage {
    Unknown = 0,
    GpuOnly = 1,
    CpuOnly = 2,
    CpuToGpu = 3,
    GpuToCpu = 4,
}

#[repr(C)]
pub struct PoolCreateInfo {
    pub memory_type_index: u32,
    pub flags: PoolCreateFlags,
    pub block_size: vk::DeviceSize,
    pub min_block_count: usize,
    pub max_block_count: usize,
    pub frame_in_use_count: u32,
}

#[repr(C)]
pub struct PoolStats {
    pub size: vk::DeviceSize,
    pub unused_size: vk::DeviceSize,
    pub allocation_count: usize,
    pub unused_range_count: usize,
    pub unused_range_size_max: vk::DeviceSize,
    pub block_count: usize,
}

#[repr(C)]
pub struct AllocationInfo {
    pub memory_type: u32,
    pub device_memory: vk::DeviceMemory,
    pub offset: vk::DeviceSize,
    pub size: vk::DeviceSize,
    pub mapped_data: *mut c_void,
    pub user_data: *mut c_void,
}

#[repr(C)]
pub struct AllocationCreateInfo {
    pub flags: AllocationCreateFlags,
    pub usage: MemoryUsage,
    pub required_flags: vk::MemoryPropertyFlags,
    pub preferred_flags: vk::MemoryPropertyFlags,
    pub memory_type_bits: u32,
    pub pool: Pool,
    pub user_date: *mut c_void,
}

#[repr(C)]
pub struct DefragmentationInfo {
    pub max_bytes_to_move: vk::DeviceSize,
    pub max_allocations_to_move: u32,
}

#[repr(C)]
pub struct DefragmentationStats {
    pub bytes_moved: vk::DeviceSize,
    pub bytes_freed: vk::DeviceSize,
    pub allocations_moved: u32,
    pub device_memory_blocks_freed: u32,
}

extern "C" {
    pub fn vmaCreateAllocator(
        create_info: *const AllocatorCreateInfo,
        allocator: *mut Allocator,
    ) -> vk::Result;

    pub fn vmaDestroyAllocator(allocator: Allocator);

    pub fn vmaGetPhysicalDeviceProperties(
        allocator: Allocator,
        physical_device_properties: *mut *const vk::PhysicalDeviceProperties,
    );

    pub fn vmaGetMemoryProperties(
        allocator: Allocator,
        physical_device_memory_properties: *mut *const vk::PhysicalDeviceMemoryProperties,
    );

    pub fn vmaGetMemoryTypeProperties(
        allocator: Allocator,
        memory_type_index: u32,
        flags: *mut vk::MemoryPropertyFlags,
    );

    pub fn vmaSetCurrentFrameIndex(allocator: Allocator, frame_index: u32);

    pub fn vmaCalculateStats(allocator: Allocator, stats: *mut Stats);

    pub fn vmaFindMemoryTypeIndex(
        allocator: Allocator,
        memory_type_bits: u32,
        allocation_create_info: *const AllocationCreateInfo,
        memory_type_index: *mut u32,
    ) -> vk::Result;

    pub fn vmaFindMemoryTypeIndexForBufferInfo(
        allocator: Allocator,
        buffer_create_info: *const vk::BufferCreateInfo,
        allocation_create_info: *const AllocationCreateInfo,
        memory_type_index: *mut u32,
    ) -> vk::Result;

    pub fn vmaFindMemoryTypeIndexForImageInfo(
        allocator: Allocator,
        image_create_info: *const vk::ImageCreateInfo,
        allocation_create_info: *const AllocationCreateInfo,
        memory_type_index: *mut u32,
    ) -> vk::Result;

    pub fn vmaCreatePool(
        allocator: Allocator,
        create_info: *const PoolCreateInfo,
        pool: *mut Pool,
    ) -> vk::Result;

    pub fn vmaDestroyPool(allocator: Allocator, pool: Pool);

    pub fn vmaGetPoolStats(allocator: Allocator, pool: Pool, pool_stats: *mut PoolStats);

    pub fn vmaMakePoolAllocationsLost(
        allocator: Allocator,
        pool: Pool,
        lost_allocation_count: *mut usize,
    );

    pub fn vmaCheckPoolCorrupion(allocator: Allocator, pool: Pool) -> vk::Result;

    pub fn vmaAllocateMemory(
        allocator: Allocator,
        memory_requirements: *const vk::MemoryRequirements,
        create_info: *const AllocationCreateInfo,
        allocation: *mut Allocation,
        allocation_info: *mut AllocationInfo,
    ) -> vk::Result;

    pub fn vmaAllocateMemoryForBuffer(
        allocator: Allocator,
        buffer: vk::Buffer,
        create_info: *const AllocationCreateInfo,
        allocation: *mut Allocation,
        allocation_info: *mut AllocationInfo,
    ) -> vk::Result;

    pub fn vmaAllocateMemoryForImage(
        allocator: Allocator,
        image: vk::Image,
        create_info: *const AllocationCreateInfo,
        allocation: *mut Allocation,
        allocation_info: *mut AllocationInfo,
    ) -> vk::Result;

    pub fn vmaGetAllocationInfo(
        allocator: Allocator,
        allocation: Allocation,
        allocation_info: *mut AllocationInfo,
    );

    pub fn vmaTouchAllocation(allocator: Allocator, allocation: Allocation) -> bool;

    pub fn vmaSetAllocationUserData(
        allocator: Allocator,
        allocation: Allocation,
        user_data: *mut c_void,
    );

    pub fn vmaCreateLostAllocation(allocator: Allocation, allocation: *mut Allocation);

    pub fn vmaMapMemory(
        allocator: Allocator,
        allocation: Allocation,
        data: *mut *mut c_void,
    ) -> vk::Result;

    pub fn vmaUnmapMemory(allocator: Allocator, allocation: Allocation);

    pub fn vmaFlushAllocation(
        allocator: Allocator,
        allocation: Allocation,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
    );

    pub fn vmaInvalidateAllocation(
        allocator: Allocator,
        allocation: Allocation,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
    );

    pub fn vmaCheckCorruption(allocator: Allocator, memory_type_bits: u32) -> vk::Result;

    pub fn vmaDefragment(
        allocator: Allocator,
        allocations: *mut Allocation,
        allocation_count: usize,
        allocations_changed: *mut bool,
        defragmentation_info: *const DefragmentationInfo,
        defragmentation_stats: *mut DefragmentationStats,
    ) -> vk::Result;

    pub fn vmaBindBufferMemory(
        allocator: Allocator,
        allocation: Allocation,
        buffer: vk::Buffer,
    ) -> vk::Result;

    pub fn vmaBindImageMemory(
        allocator: Allocator,
        allocation: Allocation,
        image: vk::Image,
    ) -> vk::Result;

    pub fn vmaCreateBuffer(
        allocator: Allocator,
        buffer_create_info: *const vk::BufferCreateInfo,
        allocation_create_info: *const AllocationCreateInfo,
        buffer: *mut vk::Buffer,
        allocation: *mut Allocation,
        allocation_info: *mut AllocationInfo,
    ) -> vk::Result;

    pub fn vmaDestroyBuffer(allocator: Allocator, buffer: vk::Buffer, allocation: Allocation);

    pub fn vmaCreateImage(
        allocator: Allocator,
        image_create_info: *const vk::ImageCreateInfo,
        allocation_create_info: *const AllocationCreateInfo,
        image: *mut vk::Image,
        allocation: *mut Allocation,
        allocation_info: *mut AllocationInfo,
    ) -> vk::Result;

    pub fn vmaDestroyImage(allocator: Allocator, image: vk::Image, allocation: Allocation);
}
