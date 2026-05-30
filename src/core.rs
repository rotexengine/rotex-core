use std::ffi::CStr;

use ash::vk;

use crate::device::Adapter;
use crate::error::{ErrorKind, Error, Severity};

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    if p_callback_data.is_null() {
        return vk::FALSE;
    }

    let message = unsafe { CStr::from_ptr((*p_callback_data).p_message) };
    let severity = format!("{:?}", message_severity).to_lowercase();
    let ty = format!("{:?}", message_type).to_lowercase();
    println!("[Debug][{}][{}] {:?}", severity, ty, message);
    vk::FALSE
}

#[derive(Debug, Clone, Copy)]
pub struct InstanceOptions {
    pub enable_validation: bool,
    pub enable_debug_utils: bool,
    pub message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    pub message_type: vk::DebugUtilsMessageTypeFlagsEXT,
}

impl Default for InstanceOptions {
    fn default() -> Self {
        let enable_validation = cfg!(debug_assertions);
        Self {
            enable_validation,
            enable_debug_utils: enable_validation,
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        }
    }
}

pub struct DebugMessenger {
    debug_utils: ash::ext::debug_utils::Instance,
    messenger: vk::DebugUtilsMessengerEXT,
}

impl DebugMessenger {
    fn new(
        entry: &ash::Entry,
        instance: &ash::Instance,
        severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    ) -> Result<Self, Error> {
        let debug_utils = ash::ext::debug_utils::Instance::new(entry, instance);
        let create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(severity)
            .message_type(message_type)
            .pfn_user_callback(Some(vulkan_debug_utils_callback));

        let messenger = unsafe { debug_utils.create_debug_utils_messenger(&create_info, None) }
            .map_err(|err| Error {
                kind: ErrorKind::Vulkan(err),
                severity: Severity::Fatal,
            })?;

        Ok(Self {
            debug_utils,
            messenger,
        })
    }

    pub fn destroy(self) {
        unsafe {
            self.debug_utils
                .destroy_debug_utils_messenger(self.messenger, None);
        }
    }
}

pub struct Instance {
    pub(crate) entry: ash::Entry,
    pub(crate) instance: ash::Instance,
}

impl Instance {
    pub fn new(extensions: &[*const i8]) -> Result<(Self, Option<DebugMessenger>), Error> {
        Self::new_with_options(&InstanceOptions::default(), extensions)
    }

    pub fn new_with_options(
        options: &InstanceOptions,
        extensions: &[*const i8],
    ) -> Result<(Self, Option<DebugMessenger>), Error> {
        let entry = ash::Entry::linked();
        let appname = std::ffi::CString::new("Rotex").unwrap();
        let enginename = std::ffi::CString::new("Rotex").unwrap();
        let app_info = vk::ApplicationInfo::default()
            .application_name(&appname)
            .engine_name(&enginename)
            .application_version(vk::make_api_version(0, 0, 1, 0))
            .engine_version(vk::make_api_version(0, 0, 1, 0))
            .api_version(vk::make_api_version(0, 1, 4, 0));
        let layer_names: Vec<std::ffi::CString> = if options.enable_validation {
            vec![std::ffi::CString::new("VK_LAYER_KHRONOS_validation").unwrap()]
        } else {
            Vec::new()
        };
        let layer_name_pointers: Vec<*const i8> = layer_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();
        let mut extension_name_pointers = extensions.to_vec();
        if options.enable_debug_utils {
            extension_name_pointers.push(ash::ext::debug_utils::NAME.as_ptr());
        }

        let mut debugcreateinfo = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(options.message_severity)
            .message_type(options.message_type)
            .pfn_user_callback(Some(vulkan_debug_utils_callback));

        let mut instance_create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_layer_names(layer_name_pointers.as_ref())
            .enabled_extension_names(extension_name_pointers.as_ref());
        if options.enable_debug_utils {
            instance_create_info = instance_create_info.push_next(&mut debugcreateinfo);
        }
        if options.enable_validation {
            dbg!(&instance_create_info);
        }
        let instance =
            unsafe { entry.create_instance(&instance_create_info, None) }.map_err(|err| {
                Error {
                    kind: ErrorKind::Vulkan(err),
                    severity: Severity::Fatal,
                }
            })?;

        let debug_messenger = if options.enable_debug_utils {
            let messenger = DebugMessenger::new(
                &entry,
                &instance,
                options.message_severity,
                options.message_type,
            );
            match messenger {
                Ok(messenger) => Some(messenger),
                Err(err) => {
                    unsafe {
                        instance.destroy_instance(None);
                    }
                    return Err(err);
                }
            }
        } else {
            None
        };

        Ok((Self { entry, instance }, debug_messenger))
    }

    pub fn entry(&self) -> &ash::Entry {
        &self.entry
    }

    pub fn instance(&self) -> &ash::Instance {
        &self.instance
    }

    pub fn enumerate_adapters(&self) -> Vec<Adapter> {
        let devices = unsafe { self.instance.enumerate_physical_devices() }.unwrap_or_else(|err| {
            eprintln!("failed to enumerate physical devices: {err:?}");
            Vec::new()
        });

        devices
            .into_iter()
            .map(|handle| {
                let props = unsafe { self.instance.get_physical_device_properties(handle) };
                let name = unsafe { CStr::from_ptr(props.device_name.as_ptr()) }
                    .to_string_lossy()
                    .into_owned();
                Adapter::new(handle, name, props.device_type, props.limits)
            })
            .collect()
    }

    pub fn destroy(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}
