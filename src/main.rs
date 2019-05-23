#![recursion_limit = "128"]
#[macro_use]
mod convenience;
mod auth;
mod freedesktop;
mod message;
mod parameterization;
mod safe;
mod webkit;

use parameterization::Config;
use webkit::{UserContentManagerHelpers, WebViewHelpers};

use clap::App;
use const_c_str::c_str;
use gdk::{Cursor, CursorType, ScreenExt, WindowExt};
use gtk::{ContainerExt, Continue, GtkWindowExt, Inhibit, WidgetExt, Window, WindowType};
use pam::Converse;
use webkit2gtk::{
    ContextMenuExt, SettingsExt, UserContentManager, WebContext, WebInspectorExt, WebView,
    WebViewExt, WebViewExtManual,
};

use users;
use users::os::unix::UserExt;

use nix::unistd::{chdir, gethostname};

use std::cell::Cell;
use std::collections::HashMap;
use std::ffi::CString;
use std::os::unix::process::CommandExt;
use std::rc::Rc;
use std::sync::Mutex;

#[derive(Debug)]
enum ProgramError {
    Config(parameterization::ConfigError),
    Io(std::io::Error),
    GenericError(String),
    XServerQuit,
}

const JS_NUMBER_MASK: u64 = (1 << 53) - 1;

fn kill_x<'a, T>(x: &'a mut Option<std::process::Child>) -> impl FnOnce(T) -> T + 'a {
    |e| {
        for x in x {
            println!("Killing x server...");
            x.kill().ok();
            x.wait().expect("X server to cleanly exit");
            println!("Done");
        }
        e
    }
}

fn main() -> Result<(), ProgramError> {
    let app = parameterization::WebDMApp::from(
        App::new("WebDM")
            .version(clap::crate_version!())
            .author(clap::crate_authors!())
            .about(clap::crate_description!()),
    );

    let config = Config::from(app)?;

    let display_cstr = CString::new(config.xorg.display.clone())
        .expect("Display string should not contain any nul bytes");

    let mut x = if config.create_x_server {
        println!("Creating x server");
        let mut x = safe::x11::start_x_server(&config.xorg.display, &config.xorg.vt)
            .map_err(ProgramError::Io)?;

        while !safe::x11::poll_for_x_available(&mut x, display_cstr.as_c_str())? {}
        Some(x)
    } else {
        None
    };

    println!("Setting DISPLAY env to {:#?}", config.xorg.display);
    safe::libc::setenv(c_str!("DISPLAY"), display_cstr.as_c_str()).map_err(kill_x(&mut x))?;

    println!("Starting login greeter");
    let (mut wm, pam) = webkit(config).map_err(kill_x(&mut x))?;

    println!("X Session started");

    wm.wait()
        .map_err(|e| {
            ProgramError::GenericError(format!("Error while waiting for x session to stop: {}", e))
        })
        .map_err(kill_x(&mut x))?;

    drop(pam);

    println!("X Session exited");
    kill_x(&mut x)(());
    println!("Finished");
    Ok(())
}

macro_rules! catch {
    ($cell:ident, $result:expr, $or_else:tt) => {
        match $result {
            Ok(v) => v,
            Err(e) => {
                $cell.set(Some(Err(e)));
                gtk::main_quit();
                $or_else
            }
        }
    };
    ($cell:ident, $result:expr) => {
        match $result {
            Ok(v) => v,
            Err(e) => {
                $cell.set(Some(Err(e)));
                gtk::main_quit();
            }
        }
    };
}

fn webkit(
    config: Config,
) -> Result<(std::process::Child, pam::Authenticator<'static, pam::PasswordConv>), ProgramError> {
    let theme_path = config.theme.path.clone();
    let display = config.xorg.display;
    let secure = !config.theme.allow_external_resources;
    let default_session_name = config.session.default;
    let session_path = config.session.path;
    let hide_users = config.users.hide;
    let debug = config.theme.debug;

    let (callbacks, mut authenticator) = auth::Auth::create(
        display,
        u8::from_str_radix(config.xorg.vt.trim_start_matches("vt"), 10)
            .map_err(|e| ProgramError::GenericError(format!("Could not parse vt string: {}", e)))?,
    )
    .map_err(|e| {
        ProgramError::GenericError(format!("Could not create PAM authenticator: {:?}", e))
    })?;

    let send_auth = authenticator.sender();

    let http_server = safe::libc::run_in_process(|| {
        rouille::start_server("localhost:8742", move |request| {
            let response = rouille::match_assets(&request, &theme_path);
            if response.is_success() {
                return response.with_no_cache();
            }

            rouille::Response::empty_404().with_no_cache()
        });
    })
    .map_err(|e| {
        ProgramError::GenericError(format!("Could not fork to create http server: {}", e))
    })?;

    let gtk_proc = safe::libc::return_from_process::<Result<freedesktop::Entry, String>, _>(move || {
        convenience::maybe(move || {
            let send_auth = Rc::new(send_auth);

            gtk::init()
                .map_err(|e| format!("Failed to init GTK: {}", e))?;

            let ret: Rc<Cell<Option<_>>> = Rc::new(Cell::new(None));

            let context = WebContext::get_default().ok_or_else(|| {
                format!("Failed to get webkit context")
            })?;

            let scripts = UserContentManager::new();

            let webview = Rc::new(WebView::new_with_context_and_user_content_manager(
                &context, &scripts,
            ));

            let callback_sym = Rc::new(convenience::hash(std::time::SystemTime::now()).to_string());

            let mut default_session = None;
            let mut sessions = vec![];

            let entries: Mutex<HashMap<_, _>> = Mutex::new({
                match std::fs::read_dir(session_path) {
                    Err(_) => HashMap::new(),
                    Ok(entries) => entries
                        .into_iter()
                        .filter_map(|entry| {
                            entry.ok().and_then(|entry| {
                                let key = entry.path();
                                match freedesktop::Entry::parse(&key) {
                                    Err(e) => {
                                        println!(
                                            "Error during parsing of {:#?}: {:?}",
                                            entry.path(),
                                            e
                                        );
                                        None
                                    }
                                    Ok(entry) => {
                                        if entry.typ == freedesktop::EntryType::Application {
                                            let key = convenience::hash(
                                                key.to_string_lossy().into_owned(),
                                            ) & JS_NUMBER_MASK;

                                            let new_default = default_session_name
                                                .as_ref()
                                                .map(|default| default == &entry.name)
                                                .unwrap_or(false);

                                            let session = message::Session {
                                                key,
                                                name: entry.name.clone(),
                                                comment: entry
                                                    .comment
                                                    .as_ref()
                                                    .cloned()
                                                    .unwrap_or_else(|| "".to_string()),
                                            };

                                            if new_default {
                                                default_session = Some(session.clone());
                                            };

                                            sessions.push(session);

                                            Some((key, entry))
                                        } else {
                                            None
                                        }
                                    }
                                }
                            })
                        })
                        .collect(),
                }
            });

            let default_session = default_session.or_else(|| sessions.iter().next().cloned());

            scripts.add_onload_script(&format!(
                "{}({});",
                include_str!("script.js"),
                serde_json::json!({
                    // "can_hibernate": null,
                    // "can_restart": null,
                    // "can_shutdown": null,
                    // "can_suspend": null,
                    "default_session": default_session,
                    "hide_users": hide_users,
                    "hostname": gethostname(&mut [0u8; 1024]).ok(),
                    // "lock_hint": null,
                    "sessions": sessions,
                    "users": if hide_users { serde_json::json!([]) } else { serde_json::json!([]) },
                    "callback_secret": *callback_sym,
                    "debug": debug,
                    "secure": secure,
                })
            ));

            gtk::idle_add(clone!(ret, webview in move || {
                match callbacks.try_recv() {
                    Err(e) => {
                        if let ipc_channel::ErrorKind::Io(ref e) = *e {
                            if let std::io::ErrorKind::WouldBlock = e.kind() {
                                return Continue(true);
                            }
                        }

                        ret.set(Some(Err(
                            format!("Attempted to read from callback channel, but failed: {}", e)
                        )));

                        gtk::main_quit();
                        Continue(false)
                    },
                    Ok(call) => {
                        webview.respond(&callback_sym, call.id, call.message);
                        Continue(true)
                    }
                }
            }));

            scripts.register_message::<message::Callback<message::Session>, _>(
                "open_session",
                clone!(ret, send_auth in move |message| {
                    // TODO: Answer even if not Ok(_)
                    if let Ok(message) = message {
                        if let Some(entry) = entries
                                .lock()
                                .expect("Entries mutex to be un-poisoned")
                                .remove(&message.data.key) {
                            ret.set(Some(send_auth.send(auth::request(message.id, auth::Request::OpenSession))
                                .map(|_| entry)
                                .map_err(|e| format!("PAM channel closed, but login was attempted: {}", e))));

                            gtk::main_quit();
                        } else {
                            eprintln!("Could not find entry: {}", message.data.key);
                        }
                    }
                }),
            );

            scripts.register_message::<message::Callback<message::Login>, _>(
                "auth",
                clone!(ret, send_auth in move |message| {
                    // TODO: Answer even if not Ok(_)
                    if let Ok(message) = message {
                        catch!(ret, send_auth.send(auth::request(message.id, auth::Request::Login {
                            username: message.data.username,
                            password: message.data.password,
                        })).map_err(|e| format!("PAM channel closed, but login was attempted: {}", e)), {
                            gtk::main_quit();
                        });
                    };
                }),
            );

            scripts.register_message::<message::Callback<message::Exit>, _>("exit", move |message| {
                println!("Exit handler");
                if let Ok(_) = message {
                    println!("Ok message");
                    gtk::main_quit();
                };
            });

            webview.connect_context_menu(move |_, menu, _, _| {
                menu.remove_all();

                return true;
            });

            if secure {
                webview.only_accept_from("localhost", 8742);
            }

            webview.load_uri(&format!("http://localhost:8742/{}", "index.html"));

            let window = Window::new(WindowType::Toplevel);

            window.add(&*webview);

            let settings = WebViewExt::get_settings(&*webview).ok_or_else(|| {
                "Failed to get webkit settings".to_owned()
            })?;

            if debug {
                settings.set_enable_developer_extras(true);

                let inspector = webview.get_inspector().ok_or_else(|| {
                    "Failed to get webkit inspector".to_owned()
                })?;

                inspector.show();
            }

            window.show_all();

            let gdk_window = window
                .get_window()
                .ok_or_else(|| "Failed to get GDK window".to_owned())?;
            let display = window
                .get_display()
                .ok_or_else(|| "Failed to get GDK display".to_owned())?;
            let screen = window
                .get_screen()
                .ok_or_else(|| "Failed to get GDK screen".to_owned())?;

            gdk_window.set_cursor(Some(&Cursor::new_for_display(&display, CursorType::Arrow)));
            window.resize(screen.get_width(), screen.get_height());

            window.connect_delete_event(move |_, _| {
                gtk::main_quit();
                Inhibit(false)
            });

            gtk::main();
            println!("GTK main finished");
            webview.destroy();
            window.destroy();

            println!("GTK process finished");
            ret.replace(None).unwrap_or_else(|| Err("Unexpected program exit".into()))
        })
    }).map_err(|e| {
        ProgramError::GenericError(format!("Could not fork to create gtk process: {}", e))
    })?;

    println!("Waiting for GTK process to exit...");

    let wm = loop {
        if let Ok(true) = safe::libc::is_alive(http_server) {
        } else {
            return Err(ProgramError::GenericError(format!(
                "HTTP server died unexpectedly"
            )));
        }

        if let Err(err) = authenticator.drain() {
            eprintln!("PAM error: {:?}", err);
        }

        if let Some(wm_entry) = gtk_proc.poll_finished().map_err(|e| {
            ProgramError::GenericError(format!("Could not wait for GTK process: {:?}", e))
        })? {
            break wm_entry
                .map_err(|e| {
                    ProgramError::GenericError(format!(
                        "IPC error between GTK process and main process: {}",
                        e
                    ))
                })?
                .map_err(|msg| ProgramError::GenericError(msg))?;
        }
    };

    let env = authenticator
        .pam()
        .environment()
        .ok_or(ProgramError::GenericError(
            "Could not get PAM environment".into(),
        ))?;

    let user = users::get_user_by_name(authenticator.pam().handler().username()).ok_or(
        ProgramError::GenericError("Could not find user in user database".into()),
    )?;

    println!("Spawning wm");
    let spawned = unsafe {
        std::process::Command::new(wm.exec.as_ref().unwrap_or(&wm.name))
            .uid(user.uid())
            .gid(user.primary_group_id())
            .env_clear()
            .envs(env.iter().filter_map(|name_value| {
                let mut parts = name_value.to_str().ok()?.splitn(2, '=');

                Some((parts.next()?, parts.next()?))
            }))
            .pre_exec(move || {
                chdir(user.home_dir()).map_err(|e| {
                    e.as_errno()
                        .map(From::from)
                        .unwrap_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, e))
                })
            })
            .spawn()
            .map_err(|e| ProgramError::GenericError(format!("Could not start xsession: {}", e)))?
    };

    println!("VM running");
    println!("Killing HTTP server");

    nix::sys::signal::kill(http_server, nix::sys::signal::Signal::SIGKILL)
        .map_err(|e| ProgramError::GenericError(format!("Could not kill http server: {}", e)))?;

    println!("Waiting for HTTP server to close...");
    nix::sys::wait::waitpid(http_server, None).map_err(|e| {
        ProgramError::GenericError(format!("Failed to wait for HTTP server: {}", e))
    })?;
    println!("HTTP server closed!");

    Ok((spawned, authenticator.into_pam()))
}
