use crate::error::VividResult;
use windows::Win32::UI::Shell::{
    SHQueryUserNotificationState, QUNS_ACCEPTS_NOTIFICATIONS, QUNS_PRESENTATION_MODE,
    QUNS_RUNNING_D3D_FULL_SCREEN,
};

#[no_mangle]
pub fn handler(args: &crate::foreground_watch::ForegroundWatcherEvent) -> VividResult<()> {
    let gpu = unsafe { crate::GPU.as_ref()? };
    let previous_vibrance = gpu.write().get_vibrance()?;
    log::trace!("callback args: {:#?}", args);
    // SAFETY: Data safety is ensured by the fact that the crate::GPU mutable static is wrapped in a RwLock
    let (vibrance, fullscreen_only) = unsafe { crate::CONFIG.as_ref()? }
        .vibrance_for_program(&args.process_exe)
        .map_or_else(
            // SAFETY: Data safety is ensured by the fact that the crate::GPU mutable static is wrapped in a RwLock
            || VividResult::Ok((unsafe { crate::CONFIG.as_ref()? }.default_vibrance(), false)),
            Ok,
        )?;

    let apply = if fullscreen_only {
        log::trace!("{} requires fullscreen, detecting...", args.process_exe);
        match unsafe { SHQueryUserNotificationState() } {
            Ok(notification_state) => {
                log::trace!("Found notification state: {}", notification_state);
                matches!(
                    notification_state,
                    QUNS_RUNNING_D3D_FULL_SCREEN
                        | QUNS_PRESENTATION_MODE
                        | QUNS_ACCEPTS_NOTIFICATIONS
                )
            }
            Err(_) => false,
        }
    } else {
        true
    };

    log::trace!(
        "Vibrance: old = {} / new = {} | to be applied = {}",
        previous_vibrance,
        vibrance,
        apply
    );
    if apply && vibrance != previous_vibrance {
        log::trace!("Applying new vibrance = {}", vibrance);
        gpu.write().set_vibrance(vibrance)?;
    }

    Ok(())
}
