// Copyright (c) Microsoft Corporation.
// License: MIT OR Apache-2.0

//! # Abstract
//!
//!    This driver demonstrates use of a default I/O Queue, its
//!    request start events, cancellation event, and a synchronized DPC.
//!
//!    To demonstrate asynchronous operation, the I/O requests are not completed
//!    immediately, but stored in the drivers private data structure, and a
//!    timer DPC will complete it next time the DPC runs.
//!
//!    During the time the request is waiting for the DPC to run, it is
//!    made cancellable by the call WdfRequestMarkCancelable. This
//!    allows the test program to cancel the request and exit instantly.
//!
//!    This rather complicated set of events is designed to demonstrate
//!    the driver frameworks synchronization of access to a device driver
//!    data structure, and a pointer which can be a proxy for device hardware
//!    registers or resources.
//!
//!    This common data structure, or resource is accessed by new request
//!    events arriving, the DPC that completes it, and cancel processing.
//!
//!    Notice the lack of specific lock/unlock operations.
//!
//!    Even though this example utilizes a serial queue, a parallel queue
//!    would not need any additional explicit synchronization, just a
//!    strategy for managing multiple requests outstanding.

#![no_std]
//#![cfg_attr(feature = "nightly", feature(hint_must_use))]
//#![deny(warnings)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![allow(clippy::missing_safety_doc)]


#[cfg(not(test))]
extern crate wdk_panic;

use wdk::wdf;
#[cfg(not(test))]
use wdk_alloc::WDKAllocator;
use wdk_sys::{*};
mod wdf_object_context;
mod macros;
mod driver;

use core::sync::atomic::AtomicI32;

use wdf_object_context::{wdf_declare_context_type, wdf_declare_context_type_with_name};

#[cfg(not(test))]
#[global_allocator]
static GLOBAL_ALLOCATOR: WDKAllocator = WDKAllocator;

// {1E42C23D-26FF-49A6-9ED0-022252DB6F02}
const GUID_DEVINTERFACE_INTERUSTCEPTOR: GUID = GUID {
    Data1: 0x1E42_C23Du32,
    Data2: 0x26FF,
    Data3: 0x49A6,
    Data4: [0x9E, 0xD0, 0x02, 0x22, 0x52, 0xDB, 0x6F, 0x02],
};

// Declare queue context.
//
// ====== CONTEXT SETUP ========//

// The device context performs the same job as
// a WDM device extension in the driver frameworks
pub struct DeviceContext {
    private_device_data: ULONG, // just a placeholder
}
wdf_declare_context_type!(DeviceContext);

pub struct QueueContext {
    buffer: PVOID,
    length: usize,
    timer: wdf::Timer,
    current_request: WDFREQUEST,
    current_status: NTSTATUS,
    spin_lock: wdf::SpinLock,
}
wdf_declare_context_type_with_name!(QueueContext, queue_get_context);

pub struct RequestContext {
    cancel_completion_ownership_count: AtomicI32,
}
wdf_declare_context_type_with_name!(RequestContext, request_get_context);
