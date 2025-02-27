use std::os::raw::c_void;

use super::ExecutionProvider;
use crate::{Error, ExecutionProviderDispatch, Result, SessionBuilder};

#[derive(Debug, Clone)]
pub struct OpenVINOExecutionProvider {
	device_type: Option<String>,
	device_id: Option<String>,
	num_threads: ort_sys::size_t,
	cache_dir: Option<String>,
	context: *mut c_void,
	enable_opencl_throttling: bool,
	enable_dynamic_shapes: bool,
	enable_vpu_fast_compile: bool
}

unsafe impl Send for OpenVINOExecutionProvider {}
unsafe impl Sync for OpenVINOExecutionProvider {}

impl Default for OpenVINOExecutionProvider {
	fn default() -> Self {
		Self {
			device_type: None,
			device_id: None,
			num_threads: 8,
			cache_dir: None,
			context: std::ptr::null_mut(),
			enable_opencl_throttling: false,
			enable_dynamic_shapes: false,
			enable_vpu_fast_compile: false
		}
	}
}

impl OpenVINOExecutionProvider {
	/// Overrides the accelerator hardware type and precision with these values at runtime. If this option is not
	/// explicitly set, default hardware and precision specified during build time is used.
	pub fn with_device_type(mut self, device_type: impl ToString) -> Self {
		self.device_type = Some(device_type.to_string());
		self
	}

	/// Selects a particular hardware device for inference. If this option is not explicitly set, an arbitrary free
	/// device will be automatically selected by OpenVINO runtime.
	pub fn with_device_id(mut self, device_id: impl ToString) -> Self {
		self.device_id = Some(device_id.to_string());
		self
	}

	/// Overrides the accelerator default value of number of threads with this value at runtime. If this option is not
	/// explicitly set, default value of 8 is used during build time.
	pub fn with_num_threads(mut self, num_threads: usize) -> Self {
		self.num_threads = num_threads as _;
		self
	}

	/// Explicitly specify the path to save and load the blobs, enabling model caching.
	pub fn with_cache_dir(mut self, dir: impl ToString) -> Self {
		self.cache_dir = Some(dir.to_string());
		self
	}

	/// This option is only alvailable when OpenVINO EP is built with OpenCL flags enabled. It takes in the remote
	/// context i.e the `cl_context` address as a void pointer.
	pub fn with_opencl_context(mut self, context: *mut c_void) -> Self {
		self.context = context;
		self
	}

	/// This option enables OpenCL queue throttling for GPU devices (reduces CPU utilization when using GPU).
	pub fn with_opencl_throttling(mut self) -> Self {
		self.enable_opencl_throttling = true;
		self
	}

	/// This option if enabled works for dynamic shaped models whose shape will be set dynamically based on the infer
	/// input image/data shape at run time in CPU. This gives best result for running multiple inferences with varied
	/// shaped images/data.
	pub fn with_dynamic_shapes(mut self) -> Self {
		self.enable_dynamic_shapes = true;
		self
	}

	pub fn with_vpu_fast_compile(mut self) -> Self {
		self.enable_vpu_fast_compile = true;
		self
	}

	pub fn build(self) -> ExecutionProviderDispatch {
		self.into()
	}
}

impl From<OpenVINOExecutionProvider> for ExecutionProviderDispatch {
	fn from(value: OpenVINOExecutionProvider) -> Self {
		ExecutionProviderDispatch::OpenVINO(value)
	}
}

impl ExecutionProvider for OpenVINOExecutionProvider {
	fn as_str(&self) -> &'static str {
		"OpenVINOExecutionProvider"
	}

	#[allow(unused, unreachable_code)]
	fn register(&self, session_builder: &SessionBuilder) -> Result<()> {
		#[cfg(any(feature = "load-dynamic", feature = "openvino"))]
		{
			let openvino_options = ort_sys::OrtOpenVINOProviderOptions {
				device_type: self
					.device_type
					.clone()
					.map(|x| x.as_bytes().as_ptr() as *const std::ffi::c_char)
					.unwrap_or_else(std::ptr::null),
				device_id: self
					.device_id
					.clone()
					.map(|x| x.as_bytes().as_ptr() as *const std::ffi::c_char)
					.unwrap_or_else(std::ptr::null),
				num_of_threads: self.num_threads,
				cache_dir: self
					.cache_dir
					.clone()
					.map(|x| x.as_bytes().as_ptr() as *const std::ffi::c_char)
					.unwrap_or_else(std::ptr::null),
				context: self.context,
				enable_opencl_throttling: self.enable_opencl_throttling.into(),
				enable_dynamic_shapes: self.enable_dynamic_shapes.into(),
				enable_vpu_fast_compile: self.enable_vpu_fast_compile.into()
			};
			return crate::error::status_to_result(
				crate::ortsys![unsafe SessionOptionsAppendExecutionProvider_OpenVINO(session_builder.session_options_ptr, &openvino_options as *const _)]
			)
			.map_err(Error::ExecutionProvider);
		}

		Err(Error::ExecutionProviderNotRegistered(self.as_str()))
	}
}
