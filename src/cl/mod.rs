extern crate ocl;

use std::ffi::CString;
use ocl::builders::ContextProperties;
use ocl::{core, flags};
use ocl::enums::ArgVal;

use crate::mat::kernels::Kernel;

pub struct CL {
    context: ocl::core::Context,
    program: ocl::core::Program,
    queue: ocl::core::CommandQueue
}

impl CL {
    pub fn new() -> CL {
        let src = r#"
            __kernel void crop(__global uchar* result, __global uchar* data, int x, int y, int width, int height, int channels, int data_width) {
                int new_x = get_global_id(0);
                int new_y = get_global_id(1);
                int new_index = new_y * width * channels + new_x * channels;

                int old_x = new_x + x;
                int old_y = new_y + y;
                int old_index = old_x * channels + old_y * data_width * channels;
                result[new_index] = data[old_index];
                for (int i=0;i<channels;i++) {
                    result[new_index+i] = data[old_index+i];
                } 
            }

            __kernel void convolute(__global float* result, __global float* data, __global float* kernel_array, int width, int height, int kernel_width) {
                int result_x = get_global_id(0);
                int result_y = get_global_id(1);

                int result_width = width - kernel_width + 1;

                int result_index = result_y * result_width + result_x;

                float result_value = 0.0;
                for (int ky=0;ky<kernel_width;ky++) {
                    for (int kx=0;kx<kernel_width;kx++) {
                        int data_x = result_x + kx;
                        int data_y = result_y + ky;

                        int data_index = data_y * width + data_x;
                        
                        int k_index = kx + ky * kernel_width;
                        float k_value = kernel_array[k_index];
                        float data_value = data[data_index];

                        result_value = result_value + k_value * data_value;
                    }
                }

                result[result_index] = result_value;
            }

            __kernel void to_gray(__global uchar* result, __global uchar* data, int channels) {
                int base_index = get_global_id(0);
                uchar r = (float) data[base_index * channels];
                uchar g = (float) data[base_index * channels + 1];
                uchar b = (float) data[base_index * channels + 2];
                uchar gray = (r*0.299 + g*0.587 + b*0.114);

                result[base_index] = gray;
            }

            __kernel void normalize_u8(__global float* result, __global uchar* data, float max) {
                int index = get_global_id(0);
                float value = (float) data[index];
                float v = value/max;

                result[index] = v;
            }

            __kernel void recover_u8(__global uchar* result, __global float* data, float max) {
                int index = get_global_id(0);
                float raw_value = data[index];
                if (raw_value < 0.0) {
                    raw_value = -raw_value;
                }
                uchar value = (uchar) (raw_value / 8.0 * max);

                result[index] = value;
            }
        "#;

        // (1) Define which platform and device(s) to use. Create a context,
        // queue, and program then define some dims..
        let platform_id = core::default_platform().unwrap();
        let device_ids = core::get_device_ids(&platform_id, None, None).unwrap();
        let device_id = device_ids[0];
        let context_properties = ContextProperties::new().platform(platform_id);
        let context = core::create_context(Some(&context_properties),
            &[device_id], None, None).unwrap();
        let src_cstring = CString::new(src).unwrap();
        let program = core::create_program_with_source(&context, &[src_cstring]).unwrap();
        core::build_program(&program, Some(&[device_id]), &CString::new("").unwrap(),
            None, None).unwrap();
        let queue = core::create_command_queue(&context, &device_id, None).unwrap();

        CL {context, program, queue}
    }

    pub fn cl_crop(&self, data: &[u8], raw_width: i32, x: i32, y: i32, width: i32, height: i32, channels: i32)
    -> ocl::Result<Vec<u8>>
    {
        let dims = [width as usize, height as usize, 1];
        
        let size: usize = width  as usize * height  as usize * channels as usize;

        let mut vec = vec![0u8; size];
        let buffer = unsafe {
            core::create_buffer(&self.context, flags::MEM_READ_WRITE | flags::MEM_COPY_HOST_PTR, size, Some(&vec))?
        };

        let raw_data = unsafe {
            core::create_buffer(&self.context, flags::MEM_READ_ONLY | flags::MEM_COPY_HOST_PTR, data.len(), Some(&data))?
        };

        // (3) Create a kernel with arguments matching those in the source above:
        let kernel = core::create_kernel(&self.program, "crop")?;
        core::set_kernel_arg(&kernel, 0, ArgVal::mem(&buffer))?;
        core::set_kernel_arg(&kernel, 1, ArgVal::mem(&raw_data))?;
        core::set_kernel_arg(&kernel, 2, ArgVal::scalar(&x))?;
        core::set_kernel_arg(&kernel, 3, ArgVal::scalar(&y))?;
        core::set_kernel_arg(&kernel, 4, ArgVal::scalar(&width))?;
        core::set_kernel_arg(&kernel, 5, ArgVal::scalar(&height))?;
        core::set_kernel_arg(&kernel, 6, ArgVal::scalar(&(channels as i32)))?;
        core::set_kernel_arg(&kernel, 7, ArgVal::scalar(&(raw_width as i32)))?;

        // (4) Run the kernel:
        unsafe {
            core::enqueue_kernel(&self.queue, &kernel, 2, None, &dims,
                None, None::<core::Event>, None::<&mut core::Event>)?;
        }

        // (5) Read results from the device into a vector:
        unsafe {
            core::enqueue_read_buffer(&self.queue, &buffer, true, 0, &mut vec,
                None::<core::Event>, None::<&mut core::Event>)?;
        }

        Ok(vec)
    }

    pub fn cl_to_gray(
        &self,
        data: &[u8],
        channels: usize
    ) -> ocl::Result<Vec<u8>> {
        let size: usize = data.len()/channels as usize;
        let dims = [size, 1, 1];

        let mut vec = vec![0u8; size];
        let buffer = unsafe {
            core::create_buffer(&self.context, flags::MEM_READ_WRITE | flags::MEM_COPY_HOST_PTR, size, Some(&vec))?
        };

        let raw_data = unsafe {
            core::create_buffer(&self.context, flags::MEM_READ_ONLY | flags::MEM_COPY_HOST_PTR, data.len(), Some(&data))?
        };

        // (3) Create a kernel with arguments matching those in the source above:
        let kernel = core::create_kernel(&self.program, "to_gray")?;
        core::set_kernel_arg(&kernel, 0, ArgVal::mem(&buffer))?;
        core::set_kernel_arg(&kernel, 1, ArgVal::mem(&raw_data))?;
        core::set_kernel_arg(&kernel, 2, ArgVal::scalar(&(channels as i32)))?;

        // (4) Run the kernel:
        unsafe {
            core::enqueue_kernel(&self.queue, &kernel, 2, None, &dims,
                None, None::<core::Event>, None::<&mut core::Event>)?;
        }

        // (5) Read results from the device into a vector:
        unsafe {
            core::enqueue_read_buffer(&self.queue, &buffer, true, 0, &mut vec,
                None::<core::Event>, None::<&mut core::Event>)?;
        }

        Ok(vec)
    }

    pub fn cl_normalize(&self, data: &[u8], max: f32) -> ocl::Result<Vec<f32>> {
        let mut vec = vec![0f32; data.len()];
        let dims = [data.len(), 1, 1];
        
        let result_buffer = unsafe {
            core::create_buffer(&self.context, flags::MEM_READ_WRITE | flags::MEM_COPY_HOST_PTR, data.len(), Some(&vec))?
        };

        let data_buffer = unsafe {
            core::create_buffer(&self.context, flags::MEM_READ_ONLY | flags::MEM_COPY_HOST_PTR, data.len(), Some(data))?
        };

        let kernel = core::create_kernel(&self.program, "normalize_u8")?;
        core::set_kernel_arg(&kernel, 0, ArgVal::mem(&result_buffer))?;
        core::set_kernel_arg(&kernel, 1, ArgVal::mem(&data_buffer))?;
        core::set_kernel_arg(&kernel, 2, ArgVal::scalar(&max))?;

        // Run the kernel:
        unsafe {
            core::enqueue_kernel(&self.queue, &kernel, 1, None, &dims,
                None, None::<core::Event>, None::<&mut core::Event>)?;
        }

        // Read results from the device into a vector:
        unsafe {
            core::enqueue_read_buffer(&self.queue, &result_buffer, true, 0, &mut vec,
                None::<core::Event>, None::<&mut core::Event>)?;
        }

        Ok(vec)
    }

    pub fn cl_recover(&self, data: &[f32], max: f32) -> ocl::Result<Vec<u8>> {
        let mut vec = vec![0u8; data.len()];
        let dims = [data.len(), 1, 1];
        
        let result_buffer = unsafe {
            core::create_buffer(&self.context, flags::MEM_READ_WRITE | flags::MEM_COPY_HOST_PTR, data.len(), Some(&vec))?
        };

        let data_buffer = unsafe {
            core::create_buffer(&self.context, flags::MEM_READ_ONLY | flags::MEM_COPY_HOST_PTR, data.len(), Some(data))?
        };

        let kernel = core::create_kernel(&self.program, "recover_u8")?;
        core::set_kernel_arg(&kernel, 0, ArgVal::mem(&result_buffer))?;
        core::set_kernel_arg(&kernel, 1, ArgVal::mem(&data_buffer))?;
        core::set_kernel_arg(&kernel, 2, ArgVal::scalar(&max))?;

        // Run the kernel:
        unsafe {
            core::enqueue_kernel(&self.queue, &kernel, 1, None, &dims,
                None, None::<core::Event>, None::<&mut core::Event>)?;
        }

        // Read results from the device into a vector:
        unsafe {
            core::enqueue_read_buffer(&self.queue, &result_buffer, true, 0, &mut vec,
                None::<core::Event>, None::<&mut core::Event>)?;
        }

        Ok(vec)
    }

    pub fn cl_convolute(
        &self,
        src: &[f32],
        width: usize,
        height: usize,
        convolution_kernel: &Kernel
    ) -> ocl::Result<Vec<f32>> {
        let result_width  = width - convolution_kernel.size() + 1;
        let result_height = height - convolution_kernel.size() + 1;
        let dims = [result_width, result_height, 1];

        let mut vec = vec![0.0; result_width*result_height];

        let result_buffer = unsafe {
            core::create_buffer(&self.context, flags::MEM_READ_WRITE | flags::MEM_COPY_HOST_PTR, result_width*result_height, Some(&vec))?
        };

        let data_buffer = unsafe {
            core::create_buffer(&self.context, flags::MEM_READ_ONLY | flags::MEM_COPY_HOST_PTR, src.len(), Some(&src))?
        };

        let kernel_array = convolution_kernel.flatten();
        let kernel_buffer = unsafe {
            core::create_buffer(&self.context, flags::MEM_READ_ONLY | flags::MEM_COPY_HOST_PTR, convolution_kernel.elements(), Some(&kernel_array))?
        }; 

        // (3) Create a kernel with arguments matching those in the source above:
        let kernel = core::create_kernel(&self.program, "convolute")?;
        core::set_kernel_arg(&kernel, 0, ArgVal::mem(&result_buffer))?;
        core::set_kernel_arg(&kernel, 1, ArgVal::mem(&data_buffer))?;
        core::set_kernel_arg(&kernel, 2, ArgVal::mem(&kernel_buffer))?;
        core::set_kernel_arg(&kernel, 3, ArgVal::scalar(&(width as i32)))?;
        core::set_kernel_arg(&kernel, 4, ArgVal::scalar(&(height as i32)))?;
        core::set_kernel_arg(&kernel, 5, ArgVal::scalar(&(convolution_kernel.size() as i32)))?;

        // (4) Run the kernel:
        unsafe {
            core::enqueue_kernel(&self.queue, &kernel, 2, None, &dims,
                None, None::<core::Event>, None::<&mut core::Event>)?;
        }

        // (5) Read results from the device into a vector:
        unsafe {
            core::enqueue_read_buffer(&self.queue, &result_buffer, true, 0, &mut vec,
                None::<core::Event>, None::<&mut core::Event>)?;
        }

        Ok(vec)
    }

    pub fn cl_laplation(
        &self,
        src: &[u8],
        width: usize,
        height: usize,
        convolution_kernel: &Kernel,
        channels: usize
    ) -> ocl::Result<(usize, usize, f32, Vec<u8>)> {
        let gray_data = self.cl_to_gray(src, channels)?;
        let normalized_data = self.cl_normalize(&gray_data, 255.0)?;
        let laplation_data = self.cl_convolute(&normalized_data, width, height, convolution_kernel)?;
        let recovered_data = self.cl_recover(&laplation_data, 255.0)?;
        let length = recovered_data.len();
        let mut total = 0f64;
        for pixel in &recovered_data {
            total += *pixel as f64;
        }
        let avg = total/length as f64;

        let mut variance = 0f64;
        for pixel in &recovered_data {
            variance += (*pixel as f64 - avg).powi(2);
        }

        let standard_deviation = (variance/length as f64).sqrt() as f32;

        let result_width = width - convolution_kernel.size() + 1;
        let result_height = height - convolution_kernel.size() + 1;
        Ok((result_width, result_height, standard_deviation, recovered_data))
    }
}