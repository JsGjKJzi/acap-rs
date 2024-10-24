use crate::flex::{KeyValueSet, Subscription};
use axevent_sys::{
    ax_event_handler_free, ax_event_handler_new, ax_event_handler_subscribe, AXEvent,
    AXEventHandler,
};
use glib::{translate::from_glib_full, Error as GError};
use glib_sys::{gpointer, GError};
use quote::{quote, quote_spanned};
use std::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};
use std::{
    collections::HashMap,
    ffi::{c_uint, c_void},
    ptr,
    sync::Arc,
};
use std::{ffi::CStr, time::Duration};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics, Index,
};
/// Event subscription takes a closure? Then user does whatever they want
/// Rely on From<> to convert types.
///

// pub struct EventStream<T>
// where T: Event,
// {
//     subscription: u32,
//     sender: Arc<Sender<T>>,
//     receiver: Receiver<T>,
// }

// impl<T> EventStream<T>
// where T: Event,
// {
//     pub unsafe extern "C" fn callback<E>(sub_id: c_uint, raw_event: *mut AXEvent, user_data: gpointer)
//     where E: Event,
//     {
//         let event = E::from(raw_event);
//         let tx = unsafe {
//             &mut *(user_data as *mut Sender<E>)
//         };
//         tx.send(event);
//     }

//     fn unsubscribe() {}
// }

// pub trait Event: From<*mut AXEvent>
// {
//     fn get_key_value_set(&self) -> &KeyValueSet;
//     fn get_base_key_value_set() -> Result<KeyValueSet, GError>;
// }

#[proc_macro_derive(Event)]
pub fn derive_event(input: proc_maco::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);
    let expanded = quote! {
        // The generated impl.
        impl Event for #name #ty_generics #where_clause {
            fn get_key_value_set(&self) -> &KeyValueSet {
                &self.kv_set
            }
        }
    };

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}

#[derive(Event)]
pub struct VMDEvent {
    kv_set: KeyValueSet,
}

impl VMDEvent {
    fn from_raw(raw: *mut AXEvent) -> Self {
        unsafe {
            // Converting to `*mut` is safe as long as we ensure that none of the mutable methods on
            // `KeyValueSet` are called, which we do by never handing out a mutable reference to the
            // `KeyValueSet`.
            let key_value_set = KeyValueSet::from_raw(ax_event_get_key_value_set(raw) as *mut _);
            Self { raw, key_value_set }
        }
    }

    fn new2(key_value_set: KeyValueSet, time_stamp: Option<DateTime>) -> Self {
        unsafe {
            let raw = ax_event_new2(key_value_set.raw, time_stamp.into_glib_ptr());
            Self { raw, key_value_set }
        }
    }

    fn key_value_set(&self) -> &KeyValueSet {
        &self.key_value_set
    }

    fn time_stamp2(&self) -> DateTime {
        unsafe { from_glib_none(ax_event_get_time_stamp2(self.raw)) }
    }
    fn drop(&mut self) {
        debug!("Dropping {}", any::type_name::<Self>());
        self.key_value_set.raw = ptr::null_mut();
        unsafe {
            ax_event_free(self.raw);
        }
    }
}

impl Event for VMDEvent {
    fn get_key_value_set(&self) -> &KeyValueSet {
        &self.kv_set
    }
    fn get_base_key_value_set() -> Result<KeyValueSet, GError> {
        let mut kv_set = KeyValueSet::new();
        kv_set.add_key_value::<&CStr>(c"topic1", Some(c"tnsaxis"), Some(c"VMD"))?;
        kv_set.add_key_value::<&CStr>(c"topic2", Some(c"tnsaxis"), Some(c"Camera1ProfileANY"))?;
        kv_set.add_key_value::<&CStr>(c"active", None, None)?;
        Ok(kv_set)
    }
}

impl From<*mut AXEvent> for VMDEvent {
    fn from(event: *mut AXEvent) -> Self {}
}

#[cfg(test)]
mod tests {
    use crate::flex::Handler;
    fn vmd_event_from_ax_event() {}

    fn subscribe_to_vmd_event() {
        let handler = Handler::default();
        let stream = handler.subscribe::<VMDEvent>();
        // |rx| {
        //     let event = rx.recv().unwrap();
        // }
    }
}
