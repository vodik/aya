use core::{cell::UnsafeCell, mem};

use aya_bpf_cty::c_void;

use crate::{
    bindings::{bpf_map_def, bpf_map_type::BPF_MAP_TYPE_XSKMAP, bpf_sock_ops},
    helpers::{
        bpf_map_lookup_elem, bpf_redirect_map, bpf_sk_assign, bpf_sk_redirect_map, bpf_sk_release,
        bpf_sock_map_update,
    },
    maps::PinningType,
    programs::{SkBuffContext, SkLookupContext, SkMsgContext},
    BpfContext,
};

#[repr(transparent)]
pub struct XskMap {
    def: UnsafeCell<bpf_map_def>,
}

unsafe impl Sync for XskMap {}

impl XskMap {
    pub const fn with_max_entries(max_entries: u32, flags: u32) -> XskMap {
        XskMap {
            def: UnsafeCell::new(bpf_map_def {
                type_: BPF_MAP_TYPE_XSKMAP,
                key_size: mem::size_of::<u32>() as u32,
                value_size: mem::size_of::<u32>() as u32,
                max_entries,
                map_flags: flags,
                id: 0,
                pinning: PinningType::None as u32,
            }),
        }
    }

    pub const fn pinned(max_entries: u32, flags: u32) -> XskMap {
        XskMap {
            def: UnsafeCell::new(bpf_map_def {
                type_: BPF_MAP_TYPE_XSKMAP,
                key_size: mem::size_of::<u32>() as u32,
                value_size: mem::size_of::<u32>() as u32,
                max_entries,
                map_flags: flags,
                id: 0,
                pinning: PinningType::ByName as u32,
            }),
        }
    }

    pub unsafe fn redirect_map(&self, index: u32, flags: u64) -> i64 {
        bpf_redirect_map(
            self.def.get() as *mut _,
            index as _,
            flags,
        )
    }
}
