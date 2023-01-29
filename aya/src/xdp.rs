//! TEST
use std::{mem, io};

use aya_obj::generated::{
    xdp_umem_reg, xsk_ring_cons, xsk_ring_prod, xsk_umem, XDP_UMEM_REG, XSK_UMEM__DEFAULT_FLAGS,
    XSK_UMEM__DEFAULT_FRAME_HEADROOM, XSK_UMEM__DEFAULT_FRAME_SIZE,
};
use libc::{setsockopt, SOL_XDP, SOCK_RAW, AF_XDP, socket};

struct Umem<'a> {
    fq: xsk_ring_prod,
    cq: xsk_ring_cons,
    umem: xsk_umem,
    buf: &'a mut [u8],
}

impl<'a> Umem<'a> {
    pub fn open(umem_area: &'a mut [u8]) -> Result<Self, io::Error> {
        let sock = unsafe { socket(AF_XDP, SOCK_RAW, 0) };
        if sock < 0 {
            return Err(io::Error::last_os_error());
        }

        let mr = xdp_umem_reg {
            addr: umem_area.as_ptr() as _,
            len: umem_area.len().try_into().unwrap(),
            chunk_size: XSK_UMEM__DEFAULT_FRAME_SIZE,
            headroom: XSK_UMEM__DEFAULT_FRAME_HEADROOM,
            flags: XSK_UMEM__DEFAULT_FLAGS,
        };
        // int fd =

        let ret = unsafe {
            setsockopt(
                sock,
                SOL_XDP,
                XDP_UMEM_REG as _,
                &mr as *const _ as *const _,
                mem::size_of::<xdp_umem_reg>() as u32,
            )
        };
        if ret < 0 {
            return Err(io::Error::last_os_error());
        }

        todo!()
    }
}

// struct UmemInfo(xsk_um

// impl UmemReg {
//     fn new()
// }
