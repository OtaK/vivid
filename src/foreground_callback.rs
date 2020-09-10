use crate::error::VividResult;

#[no_mangle]
pub fn handler(args: &crate::foreground_watch::ForegroundWatcherEvent) -> VividResult<()> {
    let gpu = unsafe {crate::GPU.as_ref()? };
    let previous_vibrance = gpu.read().get_vibrance()?;
    log::trace!("callback args: {:#?}", args);
    let (vibrance, fullscreen_only) = (*crate::CONFIG)
        .vibrance_for_program(&args.process_exe)
        .unwrap_or_else(|| ((*crate::CONFIG).default_vibrance(), false));

    let apply = if fullscreen_only {
        log::trace!("{} requires fullscreen, detecting...", args.process_exe);
        use winapi::um::shellapi;
        let mut notification_state: shellapi::QUERY_USER_NOTIFICATION_STATE = shellapi::QUERY_USER_NOTIFICATION_STATE::default();
        let api_result = unsafe { shellapi::SHQueryUserNotificationState(&mut notification_state) };
        if api_result == winapi::shared::winerror::S_OK {
            log::trace!("Found notification state: {}", notification_state);
            match notification_state {
                shellapi::QUNS_RUNNING_D3D_FULL_SCREEN | shellapi::QUNS_PRESENTATION_MODE | shellapi::QUNS_ACCEPTS_NOTIFICATIONS => true,
                _ => false
            }
        } else {
            false
        }
    } else {
        true
    };

    log::trace!("Vibrance: old = {} / new = {} | to be applied = {}", previous_vibrance, vibrance, apply);
    if apply && vibrance != previous_vibrance {
        log::trace!("Applying new vibrance = {}", vibrance);
        gpu.read().set_vibrance(vibrance)?;
    }

    Ok(())
}
