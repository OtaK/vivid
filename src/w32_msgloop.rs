use crate::error::{VividError, VividResult};

#[inline(always)]
pub fn read_message(msg: &mut winapi::um::winuser::MSG) -> VividResult<()> {
    let message_result = unsafe {
        winapi::um::winuser::GetMessageA(
            msg,
            winapi::shared::ntdef::NULL as _,
            0,
            0
        )
    };

    if message_result != 0 {
        return Err(VividError::message_loop_error());
    }

    Ok(())
}
