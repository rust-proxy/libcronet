pub mod async_request;
pub mod request;

#[cfg(test)]
mod request_test;

use libloading::{Library, Symbol};
use std::ffi::{c_void, CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::{Arc, OnceLock};

pub(crate) static CRONET_LIB: OnceLock<Library> = OnceLock::new();

pub fn get_lib() -> &'static libloading::Library {
    CRONET_LIB.get().expect("Cronet library not loaded")
}

pub struct CronetApi {
    _dummy: u8,
}

impl CronetApi {
    pub fn new(path: &str) -> Result<Arc<Self>, libloading::Error> {
        let lib = unsafe { Library::new(path)? };
        let _ = CRONET_LIB.set(lib);
        Ok(Arc::new(Self { _dummy: 0 }))
    }

    pub unsafe fn get_sym<T>(&self, name: &[u8]) -> Result<Symbol<'static, T>, libloading::Error> {
        let lib = CRONET_LIB.get().expect("Cronet library not loaded");
        unsafe {
            lib.get(name)
        }
    }
}

pub struct EngineParams {
    api: Arc<CronetApi>,
    ptr: *mut c_void,
}

impl EngineParams {
    pub fn new(api: Arc<CronetApi>) -> Result<Self, libloading::Error> {
        let ptr = unsafe {
            let func: Symbol<unsafe extern "C" fn() -> *mut c_void> =
                api.get_sym(b"Cronet_EngineParams_Create")?;
            func()
        };
        Ok(Self { api, ptr })
    }

    pub fn set_enable_quic(&self, enable: bool) -> Result<(), libloading::Error> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut c_void, bool)> =
                self.api.get_sym(b"Cronet_EngineParams_enable_quic_set")?;
            func(self.ptr, enable);
        }
        Ok(())
    }

    pub fn set_user_agent(&self, user_agent: &str) -> Result<(), libloading::Error> {
        let c_str = CString::new(user_agent).unwrap();
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut c_void, *const c_char)> =
                self.api.get_sym(b"Cronet_EngineParams_user_agent_set")?;
            func(self.ptr, c_str.as_ptr());
        }
        Ok(())
    }
}

impl Drop for EngineParams {
    fn drop(&mut self) {
        unsafe {
            if let Ok(func) = self
                .api
                .get_sym::<unsafe extern "C" fn(*mut c_void)>(b"Cronet_EngineParams_Destroy")
            {
                func(self.ptr);
            }
        }
    }
}

pub struct Engine {
    pub api: Arc<CronetApi>,
    pub ptr: *mut c_void,
}

impl Engine {
    pub fn new(api: Arc<CronetApi>) -> Result<Self, libloading::Error> {
        let ptr = unsafe {
            let func: Symbol<unsafe extern "C" fn() -> *mut c_void> =
                api.get_sym(b"Cronet_Engine_Create")?;
            func()
        };
        Ok(Self { api, ptr })
    }

    pub fn start_with_params(&self, params: &EngineParams) -> Result<c_int, libloading::Error> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut c_void, *mut c_void) -> c_int> =
                self.api.get_sym(b"Cronet_Engine_StartWithParams")?;
            Ok(func(self.ptr, params.ptr))
        }
    }

    pub fn get_version_string(&self) -> Result<String, libloading::Error> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut c_void) -> *const c_char> =
                self.api.get_sym(b"Cronet_Engine_GetVersionString")?;
            let ptr = func(self.ptr);
            if ptr.is_null() {
                return Ok(String::new());
            }
            Ok(CStr::from_ptr(ptr).to_string_lossy().into_owned())
        }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe {
            if let Ok(func) = self
                .api
                .get_sym::<unsafe extern "C" fn(*mut c_void)>(b"Cronet_Engine_Destroy")
            {
                func(self.ptr);
            }
        }
    }
}
