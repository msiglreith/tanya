use crate::engine::Engine;
use d3d12::{self, dxgi};
use winapi::shared::{dxgiformat, dxgitype};
use winit::os::windows::WindowExt;
use winit::{dpi::LogicalSize, Window};

pub type Frame = usize;

pub struct SwapChain {
    swap_chain: dxgi::SwapChain3,
    resources: Vec<d3d12::Resource>,
    rtv_heap: d3d12::DescriptorHeap,
    rtv_size: u32,
}

impl SwapChain {
    pub fn get_present_target(&self, idx: usize) -> (d3d12::Resource, d3d12::CpuDescriptor) {
        let initial = self.rtv_heap.start_cpu_descriptor();
        let rtv = d3d12::CpuDescriptor {
            ptr: initial.ptr + (idx * self.rtv_size as usize),
        };

        (self.resources[idx], rtv)
    }

    pub fn begin_frame(&self) -> Frame {
        unsafe { self.swap_chain.GetCurrentBackBufferIndex() as _ }
    }

    pub fn end_frame(&self) {
        unsafe {
            self.swap_chain.Present(0, 0);
        }
    }
}

impl Drop for SwapChain {
    fn drop(&mut self) {
        unsafe {
            for resource in self.resources.drain(..) {
                resource.destroy();
            }
            self.rtv_heap.destroy();
            self.swap_chain.destroy();
        }
    }
}

impl Engine {
    pub fn create_swapchain(&self, window: &Window, buffer_count: u32) -> SwapChain {
        let swap_chain = {
            let LogicalSize { width, height } = window.get_inner_size().unwrap();
            let desc = dxgi::SwapchainDesc {
                width: width as _,
                height: height as _,
                format: dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
                stereo: false,
                sample: d3d12::SampleDesc {
                    count: 1,
                    quality: 0,
                },
                buffer_usage: dxgitype::DXGI_USAGE_RENDER_TARGET_OUTPUT,
                buffer_count: buffer_count as _,
                scaling: dxgi::Scaling::Stretch,
                swap_effect: dxgi::SwapEffect::FlipDiscard,
                alpha_mode: dxgi::AlphaMode::Ignore,
                flags: 0,
            };

            let (swapchain, _) = self.factory.as_factory2().create_swapchain_for_hwnd(
                self.queue,
                window.get_hwnd() as *mut _,
                &desc,
            );
            let (swapchain3, _): d3d12::D3DResult<dxgi::SwapChain3> = unsafe { swapchain.cast() };
            unsafe {
                swapchain.destroy();
            }

            swapchain3
        };

        let (rtv_heap, _) = self.device.create_descriptor_heap(
            buffer_count,
            d3d12::descriptor::HeapType::Rtv,
            d3d12::descriptor::HeapFlags::empty(),
            0,
        );

        let rtv_size = self
            .device
            .get_descriptor_increment_size(d3d12::descriptor::HeapType::Rtv);

        let resources = {
            let initial = rtv_heap.start_cpu_descriptor();
            let desc = d3d12::descriptor::RenderTargetViewDesc::texture_2d(
                dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
                0,
                0,
            );

            (0..buffer_count)
                .map(|i| {
                    let rtv = d3d12::CpuDescriptor {
                        ptr: initial.ptr + (i * rtv_size) as usize,
                    };
                    let (resource, _) = swap_chain.as_swapchain0().get_buffer(i);
                    self.device.create_render_target_view(resource, &desc, rtv);
                    resource
                }).collect::<Vec<_>>()
        };

        SwapChain {
            swap_chain,
            resources,
            rtv_heap,
            rtv_size,
        }
    }
}
