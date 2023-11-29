use core::time::Duration;

use crate::{
    bindings,
    error::{get_errno, Error},
    prelude::{delay_until, time_since_start, Task},
    rtos::{queue, Context, SendQueue},
    select,
};

use alloc::collections::VecDeque;
use libc::c_void;
use slice_copy::copy;

pub enum link_type_e {
    E_LINK_RECEIVER,
    E_LINK_TRANSMITTER,
    E_LINK_RX,
    E_LINK_TX,
}
impl From<link_type_e> for bindings::link_type_e {
    fn from(link: link_type_e) -> Self {
        match link {
            link_type_e::E_LINK_RECEIVER => bindings::link_type_e_E_LINK_RECIEVER,
            link_type_e::E_LINK_TRANSMITTER => bindings::link_type_e_E_LINK_TRANSMITTER,
            link_type_e::E_LINK_RX => bindings::link_type_e_E_LINK_RX,
            link_type_e::E_LINK_TX => bindings::link_type_e_E_LINK_TX,
        }
    }
}

pub struct VexLink {
    port: u8,
}

impl VexLink {
    pub unsafe fn new(port: u8) -> Self {
        VexLink { port: port }
    }

    pub fn link_init(&self, link_id: *const u8, types: link_type_e) -> Result<u32, VexLinkError> {
        match unsafe { bindings::link_init(self.port, link_id, types as u32) } {
            x if x == bindings::PROS_ERR_U_ => Err(VexLinkError::from_errno()),
            x => Ok(x),
        }
    }

    pub fn link_init_override(
        &self,
        link_id: *const u8,
        types: link_type_e,
    ) -> Result<u32, VexLinkError> {
        match unsafe { bindings::link_init_override(self.port, link_id, types as u32) } {
            x if x == bindings::PROS_ERR_U_ => Err(VexLinkError::from_errno()),
            x => Ok(x),
        }
    }

    pub fn link_connected(&self) -> Result<bool, VexLinkError> {
        match unsafe { bindings::link_connected(self.port) } {
            x if x == true || x == false => Ok(x),
            _ => Err(VexLinkError::from_errno()),
        }
    }

    pub fn link_raw_receivable_size(&self) -> Result<u32, VexLinkError> {
        match unsafe { bindings::link_raw_receivable_size(self.port) } {
            bindings::PROS_ERR_U_ => Err(VexLinkError::from_errno()),
            x => Ok(x),
        }
    }

    pub fn link_raw_transmittable_size(&self) -> Result<u32, VexLinkError> {
        match unsafe { bindings::link_raw_transmittable_size(self.port) } {
            bindings::PROS_ERR_U_ => Err(VexLinkError::from_errno()),
            x => Ok(x),
        }
    }

    pub fn link_transmit_raw(&self, data: &str, data_size: u16) -> Result<u32, VexLinkError> {
        let mut ptr: [libc::c_char; 19] = Default::default();
        copy(&mut ptr, data.as_bytes());
        match unsafe {
            bindings::link_transmit_raw(self.port, ptr.as_ptr() as *mut c_void, data_size)
        } {
            bindings::PROS_ERR_U_ => Err(VexLinkError::from_errno()),
            x => Ok(x),
        }
    }

    pub fn link_receive_raw(&self, dest: &str, data_size: u16) -> Result<u32, VexLinkError> {
        let mut ptr: [libc::c_char; 19] = Default::default();
        copy(&mut ptr, dest.as_bytes());
        match unsafe {
            bindings::link_receive_raw(self.port, ptr.as_ptr() as *mut c_void, data_size)
        } {
            bindings::PROS_ERR_U_ => Err(VexLinkError::from_errno()),
            x => Ok(x),
        }
    }

    pub fn link_transmit(&self, data: &str, data_size: u16) {
        let mut ptr: [libc::c_char; 19] = Default::default();
        copy(&mut ptr, data.as_bytes());
        // Needs to take a Packet or a String
        self.queue_vex(ptr, data_size);
    }

    pub fn queue_vex(&self, ptr: [u8; 19], data_size: u16) {
        let port = self.port;
        let mut queue1: Option<(Context, SendQueue<Packet>)> = Default::default();
        queue1.get_or_insert_with(|| {
            let (send, recv) = queue(VecDeque::<Packet>::new());
            let ctx = Context::new_global();
            let ctx_cloned = ctx.clone();
            let x = Task::spawn_ext(
                "VexLink",
                bindings::TASK_PRIORITY_MAX,
                bindings::TASK_STACK_DEPTH_DEFAULT as u16,
                move || {
                    let mut delay_target = None;
                    let mut offset = 0usize;
                    let mut clear = false;
                    let mut rumble: Option<[libc::c_char; 9]> = None;
                    'main: loop {
                        let command: Option<Packet> = select! {
                            cmd = recv.select() => Some(cmd),
                            _ = delay_until(t); Some(t) = delay_target => None,
                        };

                        let check = match unsafe {
                            bindings::link_transmit(port, ptr.as_ptr() as *mut c_void, data_size)
                        } {
                            bindings::PROS_ERR_U_ => {
                                delay_target = Some(time_since_start() + Duration::from_millis(25));
                                Err(VexLinkError::from_errno())
                            }
                            x => {
                                delay_target = Some(time_since_start() + Duration::from_millis(25));
                                Ok(x)
                            }
                        };
                    }
                },
            )
            .unwrap();
            (ctx, send)
        });
    }

    pub fn link_receive(&self, dest: &str, data_size: u16) -> Result<u32, VexLinkError> {
        let mut ptr: [libc::c_char; 19] = Default::default();
        copy(&mut ptr, dest.as_bytes());
        match unsafe { bindings::link_receive(self.port, ptr.as_ptr() as *mut c_void, data_size) } {
            bindings::PROS_ERR_U_ => Err(VexLinkError::from_errno()),
            x => Ok(x),
        }
    }

    pub fn link_clear_receive_buf(&self) -> Result<u32, VexLinkError> {
        match unsafe { bindings::link_clear_receive_buf(self.port) } {
            bindings::PROS_ERR_U_ => Err(VexLinkError::from_errno()),
            x => Ok(x),
        }
    }
}
pub enum VexLinkError {
    PortOutOfRange,
    PortNotRadio,
    PortNotConnecting,
    BusyTransmitter,
    NullData,
    ProtocolError,
    UnknownInt(i32),
}

impl VexLinkError {
    fn from_errno() -> Self {
        match get_errno() {
            libc::ENXIO => Self::PortNotConnecting,
            libc::ENODEV => Self::PortNotRadio,
            libc::EBUSY => Self::BusyTransmitter,
            libc::EINVAL => Self::NullData,
            libc::EBADMSG => Self::ProtocolError,
            x => Self::UnknownInt(x),
        }
    }
}

impl From<VexLinkError> for Error {
    fn from(err: VexLinkError) -> Self {
        match err {
            VexLinkError::PortOutOfRange => Error::Custom("port is out of range".into()),
            VexLinkError::PortNotRadio => Error::Custom("the port cannot be configured as radio".into()),
            VexLinkError::PortNotConnecting => Error::Custom(
                "the sensor is calibrating, or no link is connected via the radio".into(),
            ),
            VexLinkError::BusyTransmitter => Error::Custom("The transmitter buffer is still busy with a previous transmission, and there is no room in the FIFO buffer (queue) to transmit the data.".into()),
            VexLinkError::NullData => Error::Custom("The data given is NULL".into()),
            VexLinkError::ProtocolError => Error::Custom("Protocol error related to start byte, data size, or checksum".into()),
            VexLinkError::UnknownInt(x) => Error::System(x)
        }
    }
}

enum Packet {
    data,
}
