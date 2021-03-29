use std::ops::Deref;

use crate::{
    generated::{
        bpf_attach_type::{BPF_SK_SKB_STREAM_PARSER, BPF_SK_SKB_STREAM_VERDICT},
        bpf_prog_type::BPF_PROG_TYPE_SK_SKB,
    },
    maps::{Map, SockMap},
    programs::{load_program, LinkRef, ProgAttachLink, ProgramData, ProgramError},
    sys::bpf_prog_attach,
};

#[derive(Copy, Clone, Debug)]
pub enum SkSkbKind {
    StreamParser,
    StreamVerdict,
}

#[derive(Debug)]
pub struct SkSkb {
    pub(crate) data: ProgramData,
    pub(crate) kind: SkSkbKind,
}

impl SkSkb {
    /// Loads the program inside the kernel.
    ///
    /// See also [`Program::load`](crate::programs::Program::load).
    pub fn load(&mut self) -> Result<(), ProgramError> {
        load_program(BPF_PROG_TYPE_SK_SKB, &mut self.data)
    }

    /// Returns the name of the program.
    pub fn name(&self) -> String {
        self.data.name.to_string()
    }

    pub fn attach<T: Deref<Target = Map>>(
        &mut self,
        map: &SockMap<T>,
    ) -> Result<LinkRef, ProgramError> {
        let prog_fd = self.data.fd_or_err()?;
        let map_fd = map.inner.fd_or_err()?;

        let attach_type = match self.kind {
            SkSkbKind::StreamParser => BPF_SK_SKB_STREAM_PARSER,
            SkSkbKind::StreamVerdict => BPF_SK_SKB_STREAM_VERDICT,
        };
        bpf_prog_attach(prog_fd, map_fd, attach_type).map_err(|(_, io_error)| {
            ProgramError::SyscallError {
                call: "bpf_link_create".to_owned(),
                io_error,
            }
        })?;
        Ok(self.data.link(ProgAttachLink {
            prog_fd: Some(prog_fd),
            map_fd: Some(map_fd),
            attach_type,
        }))
    }
}