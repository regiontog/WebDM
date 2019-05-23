use ipc_channel::ipc;

use serde::{de::DeserializeOwned, Serialize};
use std::ffi::CStr;

impl<'a> From<SetEnvError<'a>> for crate::ProgramError {
    fn from(err: SetEnvError<'a>) -> crate::ProgramError {
        crate::ProgramError::GenericError(format!(
            "Could not set environment variable '{:?}', {}.",
            match err {
                SetEnvError::InvalidName(name) => name,
                SetEnvError::NoMemory(name) => name,
            },
            match err {
                SetEnvError::InvalidName(_) => "invalid name",
                SetEnvError::NoMemory(_) => "out of memory",
            }
        ))
    }
}

pub(crate) enum SetEnvError<'a> {
    InvalidName(&'a CStr),
    NoMemory(&'a CStr),
}

pub(crate) fn setenv<'a>(name: &'a CStr, value: &CStr) -> Result<(), SetEnvError<'a>> {
    let ret_code = unsafe { libc::setenv(name.as_ptr(), value.as_ptr(), 1) };
    match ret_code {
        0 => Ok(()),
        libc::EINVAL => Err(SetEnvError::InvalidName(name)),
        libc::ENOMEM => Err(SetEnvError::NoMemory(name)),
        _ => unreachable!(),
    }
}

pub(crate) fn run_in_process<F: FnOnce() -> i32>(f: F) -> nix::Result<nix::unistd::Pid> {
    Ok(match nix::unistd::fork()? {
        nix::unistd::ForkResult::Child => {
            let exit_code = f();
            std::process::exit(exit_code);
        }
        nix::unistd::ForkResult::Parent { child } => child,
    })
}

pub(crate) struct ProcessWaitHandle<T: Serialize + DeserializeOwned> {
    recv: ipc::IpcReceiver<T>,
    pid: nix::unistd::Pid,
}

#[derive(Debug)]
pub(crate) enum WaitError {
    Nix(nix::Error),
    BadIpcError,
}

impl From<nix::Error> for WaitError {
    fn from(err: nix::Error) -> Self {
        WaitError::Nix(err)
    }
}

pub(crate) fn is_alive(pid: nix::unistd::Pid) -> nix::Result<bool> {
    use nix::sys::wait::{WaitPidFlag, WaitStatus};

    match nix::sys::wait::waitpid(pid, Some(WaitPidFlag::WNOHANG))? {
        WaitStatus::StillAlive => Ok(true),
        _ => Ok(false),
    }
}

impl<T> ProcessWaitHandle<T>
where
    T: Serialize + DeserializeOwned,
{
    pub(crate) fn poll_finished(&self) -> Result<Option<Result<T, ipc_channel::Error>>, WaitError> {
        use nix::sys::wait::{WaitPidFlag, WaitStatus};

        match nix::sys::wait::waitpid(self.pid, Some(WaitPidFlag::WNOHANG))? {
            WaitStatus::Exited(_pid, 0) => Ok(Some(self.recv.recv())),
            WaitStatus::Exited(_pid, _status) => Err(WaitError::BadIpcError),
            WaitStatus::StillAlive => Ok(None),
            s => {
                eprintln!("Unexpected wait result: {:?}", s);
                panic!()
            }
        }
    }

    pub(crate) fn wait(&self) -> Result<Result<T, ipc_channel::Error>, WaitError> {
        use nix::sys::wait::WaitStatus;

        match nix::sys::wait::waitpid(self.pid, None)? {
            WaitStatus::Exited(_pid, 0) => Ok(self.recv.recv()),
            WaitStatus::Exited(_pid, _status) => Err(WaitError::BadIpcError),
            s => {
                eprintln!("Unexpected wait result: {:?}", s);
                panic!()
            }
        }
    }
}

pub(crate) fn return_from_process<T: Serialize + DeserializeOwned, F: FnOnce() -> T>(
    f: F,
) -> std::io::Result<ProcessWaitHandle<T>> {
    let (send, recv) = ipc::channel::<T>()?;

    let pid = match nix::unistd::fork().map_err(|e| match e.as_errno() {
        Some(errno) => From::from(errno),
        None => std::io::Error::new(std::io::ErrorKind::Other, e),
    })? {
        nix::unistd::ForkResult::Child => {
            let value = f();
            if let Ok(_) = send.send(value) {
                std::process::exit(0);
            } else {
                std::process::exit(1);
            }
        }
        nix::unistd::ForkResult::Parent { child } => child,
    };

    Ok(ProcessWaitHandle { recv, pid })
}
