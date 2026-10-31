use std::collections::HashMap;
use std::ffi::c_void;
use std::process::Command;
use std::sync::Mutex;

#[derive(Default, Clone)]
pub struct Extra {
    pub model: Option<String>,
    pub size_in: Option<u32>,
}

// 显示器集合不变时复用上次探测结果，避免每次刷新面板都跑 system_profiler
static CACHE: Mutex<Option<(Vec<String>, HashMap<String, Extra>)>> = Mutex::new(None);

pub fn extras(displays: &[(String, String)]) -> HashMap<String, Extra> {
    let key: Vec<String> = {
        let mut k: Vec<String> = displays.iter().map(|(ctx, _)| ctx.clone()).collect();
        k.sort();
        k
    };
    if let Ok(g) = CACHE.lock() {
        if let Some((k, v)) = &*g {
            if *k == key {
                return v.clone();
            }
        }
    }
    let names = profiler_names();
    let sizes = edid_sizes();
    let mut map = HashMap::new();
    for (ctx, serial) in displays {
        let serial_num = serial.trim_start_matches('s').parse::<u64>().ok();
        map.insert(
            ctx.clone(),
            Extra {
                model: names.get(ctx).cloned(),
                size_in: serial_num.and_then(|n| sizes.get(&n).copied()),
            },
        );
    }
    if let Ok(mut g) = CACHE.lock() {
        *g = Some((key, map.clone()));
    }
    map
}

// system_profiler 的 _spdisplays_displayID 即 CGDirectDisplayID（= displayplacer 的 Contextual id），
// _name 是型号名（U2790B / Color LCD …），displayplacer 自身不提供
fn profiler_names() -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Ok(out) = Command::new("system_profiler")
        .args(["SPDisplaysDataType", "-json"])
        .output()
    else {
        return map;
    };
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(&out.stdout) else {
        return map;
    };
    collect_names(&v, &mut map);
    map
}

fn collect_names(v: &serde_json::Value, map: &mut HashMap<String, String>) {
    match v {
        serde_json::Value::Object(o) => {
            let id = o
                .get("_spdisplays_displayID")
                .and_then(|x| x.as_str())
                .map(str::to_string);
            let name = o.get("_name").and_then(|x| x.as_str()).map(str::to_string);
            if let (Some(id), Some(name)) = (id, name) {
                map.insert(id, name);
            }
            for child in o.values() {
                collect_names(child, map);
            }
        }
        serde_json::Value::Array(a) => {
            for child in a {
                collect_names(child, map);
            }
        }
        _ => {}
    }
}

// 镜像时 CGDisplayScreenSize 返回的是镜像组的假尺寸；EDID 的物理尺寸（DisplayAttributes 里
// 的 MaxHorizontal/VerticalImageSize，单位 cm）与镜像状态无关，按 serial 号对应回显示器
#[cfg(target_os = "macos")]
fn edid_sizes() -> HashMap<u64, u32> {
    use std::os::raw::c_char;

    #[link(name = "IOKit", kind = "framework")]
    extern "C" {
        fn IOServiceGetMatchingServices(main: u32, matching: *mut c_void, out: *mut u32) -> i32;
        fn IOServiceMatching(name: *const c_char) -> *mut c_void;
        fn IOIteratorNext(iterator: u32) -> u32;
        fn IOObjectRelease(object: u32) -> i32;
        fn IORegistryEntryCreateCFProperty(
            entry: u32,
            key: *const c_void,
            allocator: *const c_void,
            options: u32,
        ) -> *const c_void;
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFStringCreateWithCString(
            allocator: *const c_void,
            s: *const c_char,
            encoding: u32,
        ) -> *const c_void;
        fn CFDictionaryGetValue(dict: *const c_void, key: *const c_void) -> *const c_void;
        fn CFNumberGetValue(number: *const c_void, the_type: u32, value: *mut i64) -> u8;
        fn CFRelease(object: *const c_void);
    }

    const K_CF_ENCODING_UTF8: u32 = 0x0800_0100;
    const K_CF_NUMBER_SINT64: u32 = 4;

    unsafe fn cf_key(s: &str) -> *const c_void {
        let c = std::ffi::CString::new(s).unwrap_or_default();
        CFStringCreateWithCString(std::ptr::null(), c.as_ptr(), K_CF_ENCODING_UTF8)
    }

    unsafe fn dict_num(dict: *const c_void, key: &str) -> Option<i64> {
        let k = cf_key(key);
        let v = CFDictionaryGetValue(dict, k);
        CFRelease(k);
        if v.is_null() {
            return None;
        }
        let mut n: i64 = 0;
        if CFNumberGetValue(v, K_CF_NUMBER_SINT64, &mut n) == 1 {
            Some(n)
        } else {
            None
        }
    }

    let mut map = HashMap::new();
    unsafe {
        let mut iter: u32 = 0;
        let matching = IOServiceMatching(b"IOMobileFramebufferShim\0".as_ptr() as *const c_char);
        if IOServiceGetMatchingServices(0, matching, &mut iter) != 0 || iter == 0 {
            return map;
        }
        loop {
            let svc = IOIteratorNext(iter);
            if svc == 0 {
                break;
            }
            let k = cf_key("DisplayAttributes");
            let attr = IORegistryEntryCreateCFProperty(svc, k, std::ptr::null(), 0);
            CFRelease(k);
            if !attr.is_null() {
                let pk = cf_key("ProductAttributes");
                let pa = CFDictionaryGetValue(attr, pk);
                CFRelease(pk);
                let serial = if pa.is_null() {
                    None
                } else {
                    dict_num(pa, "SerialNumber")
                };
                let w = dict_num(attr, "MaxHorizontalImageSize");
                let h = dict_num(attr, "MaxVerticalImageSize");
                if let (Some(serial), Some(w), Some(h)) = (serial, w, h) {
                    if w > 0 && h > 0 {
                        let diag_cm = ((w * w + h * h) as f64).sqrt();
                        map.insert(serial as u64, (diag_cm / 2.54).round() as u32);
                    }
                }
                CFRelease(attr);
            }
            IOObjectRelease(svc);
        }
        IOObjectRelease(iter);
    }
    map
}

#[cfg(not(target_os = "macos"))]
fn edid_sizes() -> HashMap<u64, u32> {
    HashMap::new()
}
