//! TEST
use aya_obj::generated::{
    xdp_mmap_offsets, xdp_umem_reg, xsk_ring_cons, xsk_ring_prod, XDP_MMAP_OFFSETS,
    XDP_UMEM_COMPLETION_RING, XDP_UMEM_FILL_RING, XDP_UMEM_PGOFF_COMPLETION_RING,
    XDP_UMEM_PGOFF_FILL_RING, XDP_UMEM_REG, XSK_RING_CONS__DEFAULT_NUM_DESCS,
    XSK_RING_PROD__DEFAULT_NUM_DESCS, XSK_UMEM__DEFAULT_FLAGS, XSK_UMEM__DEFAULT_FRAME_HEADROOM,
    XSK_UMEM__DEFAULT_FRAME_SIZE,
};
use libc::{
    getsockopt, mmap, setsockopt, socket, AF_XDP, MAP_FAILED, MAP_POPULATE, MAP_SHARED, PROT_READ,
    PROT_WRITE, SOCK_RAW, SOL_XDP,
};
use std::{alloc, io, marker::PhantomData, mem, ptr};

/// WIP - will move into the main repo
pub fn allocate_area(len: usize) -> Box<[u8]> {
    let pagesize = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize };

    let layout = alloc::Layout::from_size_align(len, pagesize).unwrap();
    let ptr = unsafe { alloc::alloc_zeroed(layout) };

    unsafe { Box::from_raw(core::ptr::slice_from_raw_parts_mut(ptr, len)) }
}

/// WIP - sanity check, probably unneeded
pub fn check_mem_aligned(buf: &[u8]) -> usize {
    let pagesize = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize };

    let addr: usize = buf.as_ptr() as _;
    !(addr & (pagesize - 1))
}

/// WIP
pub struct RingProd(xsk_ring_prod);

/// WIP
pub struct RingCons(xsk_ring_cons);

/// WIP
pub struct Umem<'a> {
    /// WIP
    pub fq: RingProd,
    /// WIP
    pub cq: RingCons,
    phantom: PhantomData<&'a ()>,
}

impl<'a> Umem<'a> {
    /// WIP
    pub fn new(umem_area: &'a mut [u8]) -> Result<Self, io::Error> {
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

        let fill_size = XSK_RING_PROD__DEFAULT_NUM_DESCS;
        let ret = unsafe {
            setsockopt(
                sock,
                SOL_XDP,
                XDP_UMEM_FILL_RING as _,
                &fill_size as *const _ as *const _,
                mem::size_of::<u32>() as u32,
            )
        };
        if ret < 0 {
            return Err(io::Error::last_os_error());
        }

        let comp_size = XSK_RING_CONS__DEFAULT_NUM_DESCS;
        let ret = unsafe {
            setsockopt(
                sock,
                SOL_XDP,
                XDP_UMEM_COMPLETION_RING as _,
                &comp_size as *const _ as *const _,
                mem::size_of::<u32>() as u32,
            )
        };
        if ret < 0 {
            return Err(io::Error::last_os_error());
        }

        let mut off: xdp_mmap_offsets = unsafe { mem::zeroed() };
        let mut len: libc::socklen_t = mem::size_of::<xdp_mmap_offsets>() as _;
        let ret = unsafe {
            getsockopt(
                sock,
                SOL_XDP,
                XDP_MMAP_OFFSETS as _,
                &mut off as *mut _ as *mut _,
                &mut len,
            )
        };
        if ret < 0 {
            return Err(io::Error::last_os_error());
        }
        dbg!(off);

        let map = unsafe {
            mmap(
                ptr::null_mut(),
                off.fr.desc as usize + fill_size as usize * mem::size_of::<u64>(),
                PROT_READ | PROT_WRITE,
                MAP_SHARED | MAP_POPULATE,
                sock,
                XDP_UMEM_PGOFF_FILL_RING as _,
            )
        };
        if map == MAP_FAILED {
            return Err(io::Error::last_os_error());
        }

        let prod_map = map as *mut u32;

        let map = unsafe {
            mmap(
                ptr::null_mut(),
                off.cr.desc as usize + comp_size as usize * mem::size_of::<u64>(),
                PROT_READ | PROT_WRITE,
                MAP_SHARED | MAP_POPULATE,
                sock,
                XDP_UMEM_PGOFF_COMPLETION_RING as _,
            )
        };
        if map == MAP_FAILED {
            return Err(io::Error::last_os_error());
        }

        let cons_map = map as *mut u32;

        Ok(Self {
            fq: RingProd(xsk_ring_prod {
                mask: fill_size - 1,
                size: fill_size,
                producer: unsafe { prod_map.offset(off.fr.producer as _) },
                consumer: unsafe { prod_map.offset(off.fr.consumer as _) },
                flags: unsafe { prod_map.offset(off.fr.flags as _) },
                ring: unsafe { prod_map.offset(off.fr.desc as _) } as *mut _,
                cached_cons: fill_size as _,
                cached_prod: 0,
            }),
            cq: RingCons(xsk_ring_cons {
                mask: comp_size - 1,
                size: comp_size,
                producer: unsafe { cons_map.offset(off.cr.producer as _) },
                consumer: unsafe { cons_map.offset(off.cr.consumer as _) },
                flags: unsafe { cons_map.offset(off.cr.flags as _) },
                ring: unsafe { cons_map.offset(off.cr.desc as _) } as *mut _,
                cached_cons: 0,
                cached_prod: 0,
            }),
            phantom: PhantomData,
        })
    }
}

// struct UmemInfo(xsk_um

// impl UmemReg {
//     fn new()
// }
