use std::{ffi::c_void, sync::Once};

use core_foundation::error::CFError;
use objc::{
    class,
    declare::ClassDecl,
    runtime::{self, Object, Sel},
    sel, sel_impl,
};

use crate::{
    stream::sc_stream_delegate_trait::SCStreamDelegateTrait,
    utils::objc::get_concrete_from_void,
};

use super::sc_stream::SCStream;

#[repr(transparent)]
pub struct SCStreamDelegateTraitWrapper<T: SCStreamDelegateTrait>(T);

unsafe impl<T: SCStreamDelegateTrait> objc::Encode for SCStreamDelegateTraitWrapper<T> {
    fn encode() -> objc::Encoding {
        unsafe { objc::Encoding::from_str("@") }
    }
}
type DidStopWithErrorMethod = extern "C" fn(&Object, Sel, *const c_void, *const c_void);
extern "C" fn did_stop_with_error<T: SCStreamDelegateTrait>(
    this: &Object,
    _cmd: Sel,
    stream_ref: *const c_void,
    error: *const c_void,
) {
    let handler: &SCStreamDelegateTraitWrapper<T> =
        unsafe { this.get_ivar("stream_delegate_wrapper") };
    let stream = unsafe { get_concrete_from_void::<SCStream>(stream_ref) };
    let error: CFError = unsafe { get_concrete_from_void(error) };
    handler.0.did_stop_with_error(stream, error);
}

type OutputVideoEffectDidStartForStreamMethod = extern "C" fn(&Object, Sel, *const c_void);
extern "C" fn output_video_effect_did_start_for_stream<T: SCStreamDelegateTrait>(
    this: &Object,
    _cmd: Sel,
    stream_ref: *const c_void,
) {
    let handler: &SCStreamDelegateTraitWrapper<T> =
        unsafe { this.get_ivar("stream_delegate_wrapper") };
    let stream = unsafe { get_concrete_from_void::<SCStream>(stream_ref) };
    handler.0.output_video_effect_did_start_for_stream(stream);
}
type OutputVideoEffectDidStopForStreamMethod = extern "C" fn(&Object, Sel, *const c_void);
extern "C" fn output_video_effect_did_stop_for_stream<T: SCStreamDelegateTrait>(
    this: &Object,
    _cmd: Sel,
    stream_ref: *const c_void,
) {
    let handler: &SCStreamDelegateTraitWrapper<T> =
        unsafe { this.get_ivar("stream_delegate_wrapper") };
    let stream = unsafe { get_concrete_from_void::<SCStream>(stream_ref) };
    handler.0.output_video_effect_did_stop_for_stream(stream);
}

fn register<T: SCStreamDelegateTrait>() {
    let mut decl =
        ClassDecl::new("StreamDelegate", class!(NSObject)).expect("Could not register class");
    unsafe {
        decl.add_ivar::<SCStreamDelegateTraitWrapper<T>>("stream_delegate_wrapper");
        decl.add_method(
            sel!(stream:didStopWithError:),
            did_stop_with_error::<T> as DidStopWithErrorMethod,
        );
        decl.add_method(
            sel!(outputVideoEffectDidStartForStream:),
            output_video_effect_did_start_for_stream::<T>
                as OutputVideoEffectDidStartForStreamMethod,
        );
        decl.add_method(
            sel!(outputVideoEffectDidStopForStream:),
            output_video_effect_did_stop_for_stream::<T> as OutputVideoEffectDidStopForStreamMethod,
        );
        decl.register();
    }
}

pub fn get_handler<T: SCStreamDelegateTrait>(handler: T) -> *mut Object {
    static REGISTER_ONCE: Once = Once::new();
    REGISTER_ONCE.call_once(register::<T>);

    unsafe {
        let error_delegate = runtime::class_createInstance(class!(StreamDelegate), 0);
        let wrapper = SCStreamDelegateTraitWrapper(handler);
        (*error_delegate).set_ivar("stream_delegate_wrapper", wrapper);
        error_delegate
    }
}
