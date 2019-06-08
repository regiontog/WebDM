use ipc_channel::ipc::{channel, IpcReceiver, IpcSender};
use pam::Converse;
use serde::{Deserialize, Serialize};
use users::os::unix::UserExt;

static PAM_SERVICE_NAME: &str = "webdm";

pub(crate) struct Auth<'a> {
    pam: pam::Authenticator<'a, pam::PasswordConv>,
    recv: IpcReceiver<Message<Request>>,
    send: IpcSender<Message<Request>>,
    callbacks: IpcSender<Message<bool>>,
    display: String,
    vtnr: u8,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct Message<T> {
    pub(crate) id: u64,
    pub(crate) message: T,
}

#[derive(Deserialize, Serialize)]
pub(crate) enum Request {
    OpenSession,
    Login { username: String, password: String },
}

#[derive(Debug)]
pub(crate) enum AuthError {
    Pam(pam::PamError),
    Io(std::io::Error),
}

#[derive(Debug)]
pub(crate) enum DrainError {
    Pam(pam::PamError),
    FailedCallback(ipc_channel::Error),
    InvalidUsername(std::ffi::NulError),
}

impl From<std::io::Error> for AuthError {
    fn from(err: std::io::Error) -> Self {
        AuthError::Io(err)
    }
}

impl From<pam::PamError> for AuthError {
    fn from(err: pam::PamError) -> Self {
        AuthError::Pam(err)
    }
}

impl From<pam::SetCredentialsError> for DrainError {
    fn from(err: pam::SetCredentialsError) -> Self {
        match err {
            pam::SetCredentialsError::PamError(err) => DrainError::Pam(err),
            pam::SetCredentialsError::InvalidUsername(err) => DrainError::InvalidUsername(err),
        }
    }
}

impl From<ipc_channel::Error> for DrainError {
    fn from(err: ipc_channel::Error) -> Self {
        DrainError::FailedCallback(err)
    }
}

impl From<pam::PamError> for DrainError {
    fn from(err: pam::PamError) -> Self {
        DrainError::Pam(err)
    }
}

pub(crate) fn request<T>(id: u64, val: T) -> Message<T> {
    Message { id, message: val }
}

impl<'a> Auth<'a> {
    pub(crate) fn create(
        display: String,
        vtnr: u8,
    ) -> Result<(IpcReceiver<Message<bool>>, Self), AuthError> {
        let (send, recv) = channel()?;
        let (callbacks, cb_recv) = channel()?;

        Ok((
            cb_recv,
            Self {
                pam: pam::Authenticator::with_password(PAM_SERVICE_NAME)?,
                callbacks,
                recv,
                send,
                display,
                vtnr,
            },
        ))
    }

    pub fn into_pam(self) -> pam::Authenticator<'a, pam::PasswordConv> {
        self.pam
    }

    pub(crate) fn sender(&self) -> IpcSender<Message<Request>> {
        self.send.clone()
    }

    pub(crate) fn pam(&mut self) -> &mut pam::Authenticator<'a, pam::PasswordConv> {
        &mut self.pam
    }

    pub(crate) fn drain(&mut self) -> Result<ipc_channel::Error, DrainError> {
        loop {
            let result = self.recv.try_recv();

            match result {
                Ok(msg) => match msg.message {
                    Request::OpenSession => {
                        println!("Attempting to open PAM session");

                        let username = self.pam.handler().username().to_string();

                        self.pam.env("XDG_SESSION_TYPE", "x11")?;
                        self.pam.env("XDG_SESSION_CLASS", "user")?;
                        self.pam.env("XDG_VTNR", &self.vtnr.to_string())?;
                        self.pam.env("XDG_SEAT", "seat0")?;

                        let session = self.pam.open_session();

                        self.pam.env("DISPLAY", &self.display)?;
                        self.pam.env("USER", &username)?;

                        let mut opened = false;

                        match session {
                            Ok(_) => {
                                println!("Getting username");

                                println!("Looking up user");
                                if let Some(user) = users::get_user_by_name(&username) {
                                    println!("Setting PAM envs");

                                    self.pam.env("SHELL", &user.shell().to_string_lossy())?;
                                    self.pam.env("HOME", &user.home_dir().to_string_lossy())?;
                                    self.pam.env("PWD", &user.home_dir().to_string_lossy())?;

                                    opened = true;
                                } else {
                                    eprintln!(
                                        "Could not find user '{}' in user database",
                                        username
                                    );
                                }
                            }
                            Err(_) => {
                                eprintln!("Failed to open PAM session");
                            }
                        }

                        self.callbacks.send(Message {
                            id: msg.id,
                            message: opened,
                        })?;
                    }
                    Request::Login { username, password } => {
                        self.pam
                            .handler_mut()
                            .set_credentials(username.clone(), password.clone())?;

                        println!("Attempting to authenticate user '{}' with PAM", username);
                        let auth = self.pam.authenticate();

                        self.callbacks.send(Message {
                            id: msg.id,
                            message: auth.is_ok(),
                        })?;
                    }
                },
                Err(err) => return Ok(err),
            }
        }
    }
}
