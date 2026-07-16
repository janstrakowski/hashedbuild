use crate::data;

#[repr(C)]
pub enum SchemeType {
    Concrete,
    Argument,
    Call,
    Future,
}

#[repr(C)]
pub struct Scheme {
    t: SchemeType,
    u: SchemeU,
}

#[repr(C)]
pub union SchemeU {
    concrete: data::Value,
    call: Call,
    future: Future,
}

#[repr(C)]
pub struct Call {
    fun: *mut SchemeType,
    arg: *mut SchemeType,
}

type PollCb = unsafe extern "C" fn(
    *mut c_void,
    *mut Scheme,
)

#[repr(C)]
pub struct Future {
    poll_cb: PollCb,
    poll_arg: *mut c_void,
}
