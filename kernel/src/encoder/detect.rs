//! Detección de backends de hardware por plataforma.
//!
//! Estrategia:
//! - Windows: NVENC (nvEncodeAPI64.dll + GPU NVIDIA) → QuickSync
//!   (libmfxhw64.dll / libvpl.dll + iGPU Intel) → software.
//! - macOS: VideoToolbox (siempre presente) → software.
//! - Linux: NVENC (libnvidia-encode.so + /dev/nvidia*) → QuickSync
//!   (libmfxhw64.so / libvpl.so + /dev/dri) → software.
//!
//! Las funciones `has_*` son conservadoras: si dudan, devuelven
//! `false` y el orquestador usa software. Nunca panic.

use super::types::EncoderBackendKind;

/// Orden de preferencia de backends para el SO actual.
pub fn preferred_order() -> Vec<EncoderBackendKind> {
    #[cfg(target_os = "macos")]
    {
        vec![
            EncoderBackendKind::VideoToolbox,
            EncoderBackendKind::Software,
        ]
    }
    #[cfg(target_os = "windows")]
    {
        vec![
            EncoderBackendKind::Nvenc,
            EncoderBackendKind::QuickSync,
            EncoderBackendKind::Software,
        ]
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        vec![
            EncoderBackendKind::Nvenc,
            EncoderBackendKind::QuickSync,
            EncoderBackendKind::Software,
        ]
    }
}

/// ¿Hay GPU NVIDIA con encoder disponible?
///
/// Windows: `nvEncodeAPI64.dll` cargable.
/// Linux: `libnvidia-encode.so.1` o nodos `/dev/nvidia*`.
/// macOS: nunca (Apple descontinuó NVIDIA en 2019+).
pub fn has_nvenc() -> bool {
    #[cfg(target_os = "macos")]
    {
        false
    }
    #[cfg(target_os = "windows")]
    {
        library_present(&["nvEncodeAPI64.dll"])
            || env_has("REMOTE_SUPPORT_FORCE_NVENC")
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        library_present(&[
            "libnvidia-encode.so.1",
            "libnvidia-encode.so",
        ]) || std::path::Path::new("/dev/nvidia0").exists()
            || env_has("REMOTE_SUPPORT_FORCE_NVENC")
    }
}

/// ¿Hay Intel QuickSync disponible?
///
/// Windows: `libmfxhw64.dll` (Media SDK) o `libvpl.dll` (oneVPL).
/// Linux: `libmfxhw64.so.1` / `libvpl.so*` o `/dev/dri/renderD*`.
/// macOS: solo en Intel Macs viejos vía Media SDK — devolvemos false
/// porque VideoToolbox ya cubre todo el HW Apple/Intel.
pub fn has_quicksync() -> bool {
    #[cfg(target_os = "macos")]
    {
        false
    }
    #[cfg(target_os = "windows")]
    {
        library_present(&["libmfxhw64.dll", "libvpl.dll"])
            || env_has("REMOTE_SUPPORT_FORCE_QSV")
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        library_present(&[
            "libmfxhw64.so.1",
            "libmfx.so.1",
            "libvpl.so.2",
            "libvpl.so",
        ]) || std::path::Path::new("/dev/dri/renderD128").exists()
            || env_has("REMOTE_SUPPORT_FORCE_QSV")
    }
}

/// ¿Hay Apple VideoToolbox? Siempre `true` en macOS/iOS.
pub fn has_videotoolbox() -> bool {
    cfg!(target_os = "macos") || env_has("REMOTE_SUPPORT_FORCE_VTB")
}

/// Intenta cargar cada `.dll` / `.so` candidato con el loader del SO.
/// No enlaza nada en build-time: usa `LoadLibraryW` / `dlopen`.
fn library_present(candidates: &[&str]) -> bool {
    candidates.iter().any(|name| {
        #[cfg(target_os = "windows")]
        {
            load_windows_dll(name)
        }
        #[cfg(not(target_os = "windows"))]
        {
            load_posix_lib(name)
        }
    })
}

#[cfg(target_os = "windows")]
fn load_windows_dll(name: &str) -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    type HModule = *mut core::ffi::c_void;
    #[link(name = "kernel32")]
    extern "system" {
        fn LoadLibraryExW(
            lpfilename: *const u16,
            hfile: *mut core::ffi::c_void,
            dwflags: u32,
        ) -> HModule;
        fn FreeLibrary(hmodule: HModule) -> i32;
    }
    // LOAD_LIBRARY_AS_DATAFILE: no ejecuta DllMain, solo sondea.
    const LOAD_LIBRARY_AS_DATAFILE: u32 = 0x0000_0002;

    let wide: Vec<u16> = OsStr::new(name).encode_wide().chain([0]).collect();
    // SAFETY: LoadLibraryExW con AS_DATAFILE no ejecuta código de la DLL.
    let h = unsafe { LoadLibraryExW(wide.as_ptr(), core::ptr::null_mut(), LOAD_LIBRARY_AS_DATAFILE) };
    if h.is_null() {
        return false;
    }
    // SAFETY: handle válido recién obtenido.
    unsafe {
        FreeLibrary(h);
    }
    true
}

#[cfg(not(target_os = "windows"))]
fn load_posix_lib(name: &str) -> bool {
    use std::ffi::CString;
    use std::os::raw::{c_int, c_void};

    #[link(name = "dl")]
    extern "C" {
        fn dlopen(filename: *const i8, flag: c_int) -> *mut c_void;
        fn dlclose(handle: *mut c_void) -> c_int;
    }
    // RTLD_NOW | RTLD_LOCAL
    const RTLD_NOW: c_int = 0x2;
    const RTLD_LOCAL: c_int = 0;

    let Ok(cname) = CString::new(name) else {
        return false;
    };
    // SAFETY: dlopen con RTLD_NOW|RTLD_LOCAL; cerramos de inmediato.
    let h = unsafe { dlopen(cname.as_ptr(), RTLD_NOW | RTLD_LOCAL) };
    if h.is_null() {
        return false;
    }
    unsafe {
        dlclose(h);
    }
    true
}

/// Override manual para tests / CI sin GPU:
/// `REMOTE_SUPPORT_FORCE_NVENC=1`, `..._QSV=1`, `..._VTB=1`.
fn env_has(var: &str) -> bool {
    std::env::var_os(var).map(|v| v == "1" || v == "true").unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_ends_in_software() {
        let order = preferred_order();
        assert_eq!(*order.last().unwrap(), EncoderBackendKind::Software);
        assert!(order.len() >= 2);
    }

    #[test]
    fn forced_env_overrides() {
        std::env::set_var("REMOTE_SUPPORT_FORCE_NVENC", "1");
        #[cfg(target_os = "windows")]
        assert!(has_nvenc());
        std::env::remove_var("REMOTE_SUPPORT_FORCE_NVENC");
    }
}
