#![allow(non_snake_case)]

mod api;
mod config;
mod hooks;
mod il2cpp;
mod persistence;
mod plugin_api;
mod reflection;

use crate::config::init_paths;
use crate::hooks::{
    race_info_hook, team_stadium_result_hook, veteran_hook, API_HOOK_FNS, API_HOOK_ORIGS,
    MAX_API_HOOKS, ORIG_GET_RACE_TRACK_ID, ORIG_TEAM_STADIUM_RESULT, ORIG_VETERAN_APPLY,
};
use crate::il2cpp::{
    find_image_by_name, find_method_addr_by_name, get_class_from_image, init_il2cpp_methods,
    method_addr, RawIl2CppImage,
};
use crate::plugin_api::{hook, store_vtable, vtable, InitResult, Vtable};
use crate::reflection::find_methods_in_assembly_by_param;

#[no_mangle]
pub extern "C" fn hachimi_init(vtable_ptr: *const Vtable, version: i32) -> InitResult {
    if vtable_ptr.is_null() || version < 2 {
        return InitResult::Error;
    }
    store_vtable(vtable_ptr);
    init()
}

fn init() -> InitResult {
    log!("Plugin initialized.");

    if let Err(e) = init_paths() {
        log!("Failed to initialize paths: {}", e);
        return InitResult::Error;
    }

    let vt = vtable();

    unsafe {
        if !init_il2cpp_methods(|name| (vt.il2cpp_resolve_symbol)(name)) {
            log!("Failed to resolve IL2CPP scanning functions.");
            return InitResult::Error;
        }
    }

    std::thread::spawn(|| unsafe {
        install_hooks();
    });

    InitResult::Ok
}

unsafe fn install_hooks() {
    let mut target_image = find_image_by_name("umamusume");
    if target_image.is_null() {
        target_image = find_image_by_name("Assembly-CSharp");
    }
    if target_image.is_null() {
        target_image = find_image_by_name("Gallop");
    }

    if target_image.is_null() {
        log!("Error: Could not find game assembly.");
        return;
    }

    let race_info_class =
        get_class_from_image(target_image, c"Gallop".as_ptr(), c"RaceInfo".as_ptr());
    if !race_info_class.is_null() {
        let fn_ptr = find_method_addr_by_name(race_info_class, c"get_RaceTrackId".as_ptr(), 0);
        if fn_ptr != 0 {
            if let Some(orig) = hook(fn_ptr, race_info_hook as *const () as usize) {
                ORIG_GET_RACE_TRACK_ID = orig;
                log!("Hooked: Gallop.RaceInfo.get_RaceTrackId");
            }
        } else {
            log!("Failed to find get_RaceTrackId method");
        }
    } else {
        log!("Failed to find RaceInfo class");
    }

    let results =
        find_methods_in_assembly_by_param(target_image as *mut RawIl2CppImage, "TrainedChara[]");
    if results.is_empty() {
        log!("WARNING: No methods found for Veteran Characters");
    } else {
        let best_candidate = results
            .iter()
            .find(|r| {
                r.class_name.contains("WorkTrainedCharaData") && r.method_name.contains("UpdateAll")
            })
            .or_else(|| results.first());

        if let Some(result) = best_candidate {
            let fn_ptr = method_addr(result.method);
            if fn_ptr != 0 {
                if let Some(orig) = hook(fn_ptr, veteran_hook as *const () as usize) {
                    ORIG_VETERAN_APPLY = orig;
                    log!(
                        "Veteran hook installed on {}.{}",
                        result.class_name,
                        result.method_name
                    );
                } else {
                    log!("Failed to install Veteran hook");
                }
            } else {
                log!("Veteran candidate method pointer is null");
            }
        }
    }

    match api::fetch_endpoint_config() {
        Ok(configs) => {
            log!("Fetched {} endpoint config(s) from server.", configs.len());
            api::store_endpoint_configs(configs.clone());
            install_endpoint_hooks(target_image as *mut RawIl2CppImage, &configs);
        }
        Err(e) => {
            log!(
                "Failed to fetch endpoint config: {}. No API hooks installed.",
                e
            );
        }
    }

    let results =
        find_methods_in_assembly_by_param(target_image as *mut RawIl2CppImage, "CommonResponse");
    if results.is_empty() {
        log!("WARNING: No methods found taking CommonResponse parameter");
    } else {
        log!("Found {} method(s) taking CommonResponse", results.len());
        let best_candidate = results
            .iter()
            .find(|r| r.class_name.contains("TeamStadiumResult") && r.method_name == ".ctor")
            .or_else(|| results.first());

        if let Some(result) = best_candidate {
            let fn_ptr = method_addr(result.method);
            if fn_ptr != 0 {
                if let Some(orig) = hook(fn_ptr, team_stadium_result_hook as *const () as usize) {
                    ORIG_TEAM_STADIUM_RESULT = orig;
                    log!(
                        "TeamTrials hook installed on {}.{}",
                        result.class_name,
                        result.method_name
                    );
                } else {
                    log!("Failed to install TeamTrials hook");
                }
            } else {
                log!("TeamTrials candidate method pointer is null");
            }
        }
    }
}

unsafe fn install_endpoint_hooks(
    target_image: *mut RawIl2CppImage,
    endpoints: &[crate::config::EndpointConfig],
) {
    fn is_generated(s: &str) -> bool {
        s.contains('<')
    }

    let mut slot = 0;
    for endpoint in endpoints {
        if slot >= MAX_API_HOOKS {
            log!(
                "WARNING: Maximum of {} API hooks reached, skipping remaining endpoints.",
                MAX_API_HOOKS
            );
            break;
        }
        let results = find_methods_in_assembly_by_param(target_image, &endpoint.name);
        let result = results
            .iter()
            .find(|r| !is_generated(&r.method_name) && !is_generated(&r.class_name))
            .or_else(|| results.first());

        if let Some(result) = result {
            let fn_ptr = method_addr(result.method);
            if fn_ptr != 0 {
                if let Some(orig) = hook(fn_ptr, API_HOOK_FNS[slot] as usize) {
                    API_HOOK_ORIGS[slot] = orig;
                    log!(
                        "[{}] Hook installed on {}.{}",
                        endpoint.name,
                        result.class_name,
                        result.method_name
                    );
                    slot += 1;
                } else {
                    log!("[{}] Failed to install hook", endpoint.name);
                }
            } else {
                log!("[{}] Method pointer is null", endpoint.name);
            }
        } else {
            log!("[{}] WARNING: No matching method found", endpoint.name);
        }
    }
}
