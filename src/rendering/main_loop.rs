use super::*;
impl App {
    pub fn run(&mut self) -> Result<(), String> {
        let mut first_frame = true;
        let ev_loop = self.base.event_loop.take().unwrap();
        ev_loop
            .run(|ev, win| {
                win.set_control_flow(winit::event_loop::ControlFlow::Poll);
                match ev {
                    winit::event::Event::WindowEvent { event, .. } => match event {
                        winit::event::WindowEvent::Resized(size) => self.resize(size),
                        winit::event::WindowEvent::CloseRequested => win.exit(),
                        //winit::event::WindowEvent::Destroyed => todo!(),
                        //winit::event::WindowEvent::Focused(_) => todo!(),
                        //winit::event::WindowEvent::AxisMotion { device_id, axis, value } => todo!(),
                        _ => {}
                    },
                    winit::event::Event::DeviceEvent { .. } => {}
                    //winit::event::Event::Suspended => todo!(),
                    //winit::event::Event::Resumed => todo!(),
                    winit::event::Event::NewEvents(winit::event::StartCause::Poll) => {
                        if first_frame {
                            #[cfg(feature = "profiling")]
                            self.first_frame_setup();
                        }
                        self.draw_frame();
                        first_frame = false;
                    }
                    _ => {}
                }
            })
            .map_err(|e| e.to_string())
    }
    fn draw_frame(&mut self) {
        let image_index = {
            #[cfg(feature = "profiling")]
            let _a = span!(profiling::span_location!("Acquire Image"));
            let device = &mut self.device.device;
            let mut image_index = 0;
            if self.runtime.swapchain_ok {
                match unsafe {
                    self.device.swapchain_khr.acquire_next_image(
                        self.device.swapchain,
                        u64::MAX,
                        self.runtime.image_available_semaphores[self.runtime.current_frame],
                        Vk::Fence::null(),
                    )
                } {
                    Ok(index) => {
                        image_index = index.0;
                    }
                    err @ Err(e) => {
                        if e == Vk::Result::ERROR_OUT_OF_DATE_KHR {
                            self.runtime.swapchain_ok = false;
                        } else {
                            err.unwrap();
                        }
                    }
                };
            }
            if !self.runtime.swapchain_ok {
                return;
            }

            unsafe {
                device
                    .wait_for_fences(
                        &[self.runtime.render_finished_fences[self.runtime.current_frame]],
                        true,
                        u64::MAX,
                    )
                    .unwrap();
                device
                    .reset_fences(
                        &[self.runtime.render_finished_fences[self.runtime.current_frame]],
                    )
                    .unwrap();
                device.reset_command_buffer(
                    self.runtime.command_buffers[self.runtime.current_frame],
                    Vk::CommandBufferResetFlags::empty(),
                )
            }
            .unwrap();
            image_index
        };
        #[cfg(feature = "profiling")]
        if let Some(span) = self.runtime.gpu_spans[self.runtime.current_frame].take() {
            let mut buf = [0i64; 2];
            unsafe {
                self.device.device.get_query_pool_results(
                    self.runtime.gpu_timestamps,
                    self.runtime.current_frame as u32 * 2,
                    2,
                    &mut buf,
                    Vk::QueryResultFlags::TYPE_64,
                )
            }
            .unwrap();
            span.upload_timestamp(buf[0], buf[1]);
        }
        self.record_command_buffers(self.runtime.current_frame, image_index as usize);
        let render_finished_semaphore =
            [self.runtime.render_finished_semaphores[self.runtime.current_frame]];
        let image_available_semaphore =
            [self.runtime.image_available_semaphores[self.runtime.current_frame]];
        let swapchain = [self.device.swapchain];
        let image_index_ = [image_index];
        let present_info = Vk::PresentInfoKHR::builder()
            .wait_semaphores(&render_finished_semaphore)
            .swapchains(&swapchain)
            .image_indices(&image_index_);
        let stage = [Vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffer = [self.runtime.command_buffers[self.runtime.current_frame]];
        let submit_info = [Vk::SubmitInfo::builder()
            .wait_semaphores(&image_available_semaphore)
            .wait_dst_stage_mask(&stage)
            .command_buffers(&command_buffer)
            .signal_semaphores(&render_finished_semaphore)
            .build()];
        {
            #[cfg(feature = "profiling")]
            let _a = span!(profiling::span_location!("Query Submit"));
            let device = &mut self.device.device;
            unsafe {
                device.queue_submit(
                    self.device.queue,
                    &submit_info,
                    self.runtime.render_finished_fences[self.runtime.current_frame],
                )
            }
            .unwrap();
            match unsafe {
                self.device
                    .swapchain_khr
                    .queue_present(self.device.queue, &present_info)
            } {
                Err(Vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.runtime.swapchain_ok = false;
                }
                Ok(_) => (),
                e => {
                    e.unwrap();
                }
            };
            self.runtime.current_frame =
                (self.runtime.current_frame + 1) % self.device.swapchain_images.images.len();
            #[cfg(feature = "profiling")]
            self.client.frame_mark();
        }
    }
    fn record_command_buffers(&mut self, index: usize, image_index: usize) {
        #[cfg(feature = "profiling")]
        let _a = span!(profiling::span_location!("Commands start"));
        let device = &mut self.device.device;
        let begin_info = Vk::CommandBufferBeginInfo::builder()
            .flags(Vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        let cb = self.runtime.command_buffers[index];
        unsafe { device.begin_command_buffer(cb, &begin_info) }.unwrap();
        #[cfg(feature = "profiling")]
        unsafe {
            device.cmd_reset_query_pool(cb, self.runtime.gpu_timestamps, index as u32 * 2, 2);
            device.cmd_write_timestamp(
                cb,
                Vk::PipelineStageFlags::TOP_OF_PIPE,
                self.runtime.gpu_timestamps,
                index as u32 * 2,
            )
        }
        #[cfg(feature = "profiling")]
        {
            let span = self
                .runtime
                .gpu_context
                .get()
                .unwrap()
                .span(profiling::span_location!("Rendering"))
                .unwrap();
            self.runtime.gpu_spans[index] = Some(span)
        }
        let region = Vk::Rect2D {
            offset: Vk::Offset2D { x: 0, y: 0 },
            extent: self.device.swapchain_extent,
        };
        let clear_values = [
            Vk::ClearValue {
                color: Vk::ClearColorValue {
                    float32: srgb_expand([0.3921569f32, 0.58431375f32, 0.9294119f32, 1.]),
                },
            },
            Vk::ClearValue {
                depth_stencil: Vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];
        let render_pass_begin_info = Vk::RenderPassBeginInfo::builder()
            .render_pass(self.device.renderpass)
            .framebuffer(self.device.framebuffers[image_index])
            .render_area(region)
            .clear_values(&clear_values);
        unsafe {
            device.cmd_begin_render_pass(
                self.runtime.command_buffers[index],
                &render_pass_begin_info,
                Vk::SubpassContents::INLINE,
            )
        }
        unsafe {
            device.cmd_bind_pipeline(
                self.runtime.command_buffers[index],
                Vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
            )
        }
        let viewport = Vk::Viewport {
            x: 0.,
            y: 0.,
            width: self.device.swapchain_extent.width as f32,
            height: self.device.swapchain_extent.height as f32,
            max_depth: 1.,
            min_depth: 0.,
        };
        let scissor = Vk::Rect2D {
            offset: Vk::Offset2D { x: 0, y: 0 },
            extent: self.device.swapchain_extent,
        };
        unsafe {
            device.cmd_bind_vertex_buffers(
                self.runtime.command_buffers[index],
                0,
                &[self.pipeline.vertex_buffer],
                &[0],
            )
        };
        unsafe { device.cmd_set_viewport(self.runtime.command_buffers[index], 0, &[viewport]) }
        unsafe { device.cmd_set_scissor(self.runtime.command_buffers[index], 0, &[scissor]) }
        unsafe { device.cmd_draw(self.runtime.command_buffers[index], 3, 1, 0, 0) }
        unsafe { device.cmd_end_render_pass(self.runtime.command_buffers[index]) }
        #[cfg(feature = "profiling")]
        unsafe {
            device.cmd_write_timestamp(
                cb,
                Vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                self.runtime.gpu_timestamps,
                index as u32 * 2 + 1,
            )
        }
        #[cfg(feature = "profiling")]
        {
            self.runtime.gpu_spans[index].as_mut().unwrap().end_zone()
        }
        unsafe { device.end_command_buffer(self.runtime.command_buffers[index]) }.unwrap();
    }
    #[cfg(feature = "profiling")]
    fn first_frame_setup(&mut self) {
        let device = &mut self.device.device;
        let query_pool_info = Vk::QueryPoolCreateInfo::builder()
            .query_type(Vk::QueryType::TIMESTAMP)
            .query_count(1);
        let query_pool = unsafe { device.create_query_pool(&query_pool_info, None) }.unwrap();
        let fence = unsafe { device.create_fence(&Vk::FenceCreateInfo::builder(), None) }.unwrap();
        let begin_info = Vk::CommandBufferBeginInfo::builder()
            .flags(Vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        let cb = self.runtime.command_buffers[0];
        unsafe { device.begin_command_buffer(cb, &begin_info) }.unwrap();
        unsafe { device.cmd_reset_query_pool(cb, query_pool, 0, 1) };
        unsafe {
            device.cmd_write_timestamp(cb, Vk::PipelineStageFlags::ALL_COMMANDS, query_pool, 0)
        }
        unsafe { device.end_command_buffer(cb) }.unwrap();
        let cb = [cb];
        let submit_info = [Vk::SubmitInfo::builder()
            .wait_semaphores(&[])
            .wait_dst_stage_mask(&[])
            .command_buffers(&cb)
            .signal_semaphores(&[])
            .build()];
        unsafe { device.queue_submit(self.device.queue, &submit_info, fence) }.unwrap();
        unsafe { device.wait_for_fences(&[fence], true, u64::MAX) }.unwrap();
        unsafe { device.reset_fences(&[fence]) }.unwrap();
        let mut data = [0];
        unsafe {
            device.get_query_pool_results::<i64>(
                query_pool,
                0,
                1,
                &mut data,
                Vk::QueryResultFlags::TYPE_64,
            )
        }
        .unwrap();
        let gpu_context = profiling::Client::running()
            .unwrap()
            .new_gpu_context(
                Some("Main"),
                profiling::GpuContextType::Vulkan,
                data[0],
                unsafe {
                    self.base
                        .instance
                        .get_physical_device_properties(self.base.physical_device)
                        .limits
                        .timestamp_period
                },
            )
            .unwrap();
        self.runtime.gpu_context.set(gpu_context).unwrap_or(());
        unsafe { self.device.device.destroy_fence(fence, None) };
        unsafe { self.device.device.destroy_query_pool(query_pool, None) };
    }
    #[cold]
    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        #[cfg(feature = "profiling")]
        let _a = span!(profiling::span_location!("Window resize"));
        unsafe { self.device.device.device_wait_idle() }.unwrap();
        let current_image_format = device::AppDevice::get_swapchain_format(
            &self.base.surface_khr,
            &self.base.surface,
            &self.base.physical_device,
        )
        .unwrap();
        let redo_renderpass = self.device.swapchain_images.format != current_image_format.format;
        self.cleanup_swapchain(redo_renderpass);
        let device = &self.device.device;
        let swapchain = device::AppDevice::create_swapchain(
            &self.device.swapchain_khr,
            &self.base.surface_khr,
            self.base.surface,
            self.base.qu_idx,
            &self.base.physical_device,
            current_image_format,
            size,
        )
        .unwrap();
        self.device.swapchain = swapchain;
        self.device.swapchain_extent = Vk::Extent2D {
            width: size.width,
            height: size.height,
        };
        let swapchain_images =
            unsafe { self.device.swapchain_khr.get_swapchain_images(swapchain) }.unwrap();
        let swapchain_views = device::AppDevice::get_swapchain_images(
            device,
            &swapchain_images,
            current_image_format.format,
        )
        .unwrap();
        let depth_format = self.device.depth_images.format;
        let (depth_images, depth_views, depth_image_allocs) =
            device::AppDevice::create_depth_images(
                device,
                &self.device.allocator,
                depth_format,
                self.device.swapchain_extent,
                swapchain_images.len(),
                self.base.qu_idx,
            )
            .unwrap();
        if redo_renderpass {
            let renderpass = device::AppDevice::create_renderpass(
                device,
                self.device.swapchain_images.format,
                self.device.depth_images.format,
            )
            .unwrap();
            self.device.renderpass = renderpass;
        }
        let framebuffers = device::AppDevice::create_framebuffer(
            device,
            &swapchain_views,
            &depth_views,
            &self.device.renderpass,
            self.device.swapchain_extent,
        )
        .unwrap();
        let swapchain_images = device::RenderImages {
            images: swapchain_images,
            views: swapchain_views,
            format: current_image_format.format,
        };
        let depth_images = device::RenderImages {
            images: depth_images,
            views: depth_views,
            format: depth_format,
        };
        self.device.swapchain_images = swapchain_images;
        self.device.depth_images = depth_images;
        self.device.depth_image_allocs = depth_image_allocs;
        self.device.framebuffers = framebuffers;

        self.runtime.swapchain_ok = true;
    }
    pub fn cleanup_swapchain(&mut self, redo_renderpass: bool) {
        let device = &self.device.device;
        for image_view in self.device.depth_images.views.iter() {
            unsafe { device.destroy_image_view(*image_view, None) }
        }
        for image in self.device.depth_images.images.iter() {
            unsafe { device.destroy_image(*image, None) }
        }
        unsafe {
            self.device
                .swapchain_khr
                .destroy_swapchain(self.device.swapchain, None)
        };
        for allocation in self.device.depth_image_allocs.iter() {
            unsafe {
                self.device
                    .allocator
                    .deallocate(device, allocation)
                    .unwrap()
            };
        }
        for framebuffer in &self.device.framebuffers {
            unsafe { device.destroy_framebuffer(*framebuffer, None) }
        }
        if redo_renderpass {
            unsafe { device.destroy_render_pass(self.device.renderpass, None) }
        }
        for image_view in self.device.swapchain_images.views.iter() {
            unsafe { device.destroy_image_view(*image_view, None) }
        }
    }
}
