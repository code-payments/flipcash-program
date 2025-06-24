use bytemuck::{Pod, Zeroable};
use num_enum::TryFromPrimitive;
use crate::event;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum EventType {
    Unknown = 0,

    BuyEvent,
    SellEvent,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct BuyEvent {
    // todo
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct SellEvent {
    // todo
}

event!(EventType, BuyEvent);
event!(EventType, SellEvent);
