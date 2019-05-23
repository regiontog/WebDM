use glib::translate::{FromGlibPtrBorrow, ToGlibPtr};
use gtk::Cast;
use serde::Deserialize;
use webkit2gtk::{
    JavascriptResult, NavigationPolicyDecision, NavigationPolicyDecisionExt, PolicyDecisionExt,
    PolicyDecisionType, ResponsePolicyDecision, ResponsePolicyDecisionExt, URIRequestExt,
    URIResponseExt, UserContentInjectedFrames, UserContentManager, UserContentManagerExt,
    UserScript, UserScriptInjectionTime, WebView, WebViewExt,
};

#[derive(Debug)]
pub(crate) enum MessageError {
    NoContext(JavascriptResult),
    NoValue(JavascriptResult),
    NotString(JavascriptResult),
    BadString {
        result: JavascriptResult,
        err: serde_json::error::Error,
    },
}

pub(crate) trait UserContentManagerHelpers {
    fn connect_script_message_received2<F: Fn(&Self, &JavascriptResult) + 'static>(
        &self,
        name: &str,
        f: F,
    ) -> glib::SignalHandlerId;

    fn register_message<T, F: Fn(Result<T, MessageError>) + 'static>(&self, name: &str, f: F)
    where
        T: for<'a> Deserialize<'a>;

    fn add_onload_script(&self, script: &str);
}

pub(crate) trait WebViewHelpers {
    fn respond(&self, sym: &str, id: u64, js: impl core::fmt::Display);
    fn only_accept_from(&self, host: &'static str, port: u16);
}

impl UserContentManagerHelpers for UserContentManager {
    fn connect_script_message_received2<F: Fn(&Self, &JavascriptResult) + 'static>(
        &self,
        name: &str,
        f: F,
    ) -> glib::SignalHandlerId {
        unsafe {
            let f: Box<Box<Fn(&Self, &JavascriptResult) + 'static>> = Box::new(Box::new(f));
            glib::signal::connect(
                self.to_glib_none().0,
                &format!("script-message-received::{}", name),
                std::mem::transmute(script_message_received_trampoline::<Self> as usize),
                Box::into_raw(f) as *mut _,
            )
        }
    }

    fn register_message<T, F: Fn(Result<T, MessageError>) + 'static>(&self, name: &str, f: F)
    where
        T: for<'a> Deserialize<'a>,
    {
        self.connect_script_message_received2(name, move |_, message| {
            f(message
                .get_global_context()
                .ok_or(MessageError::NoContext(message.clone()))
                .and_then(|ctx| {
                    message
                        .get_value()
                        .ok_or(MessageError::NoValue(message.clone()))
                        .and_then(|value| {
                            value
                                .to_string(&ctx)
                                .ok_or(MessageError::NotString(message.clone()))
                                .and_then(|msg| {
                                    let s: Result<T, _> = serde_json::from_str(&msg);

                                    s.map_err(|err| MessageError::BadString {
                                        result: message.clone(),
                                        err: err,
                                    })
                                })
                        })
                }))
        });

        self.register_script_message_handler(name);
    }

    fn add_onload_script(&self, script: &str) {
        self.add_script(&UserScript::new(
            script,
            UserContentInjectedFrames::TopFrame,
            UserScriptInjectionTime::Start,
            &[],
            &[],
        ));
    }
}

unsafe extern "C" fn script_message_received_trampoline<P>(
    this: *mut webkit2gtk_sys::WebKitUserContentManager,
    js_result: *mut webkit2gtk_sys::WebKitJavascriptResult,
    f: glib_sys::gpointer,
) where
    P: glib::IsA<UserContentManager>,
{
    use glib::object::Downcast;
    let f: &&(Fn(&P, &webkit2gtk::JavascriptResult) + 'static) = std::mem::transmute(f);
    f(
        &UserContentManager::from_glib_borrow(this).downcast_unchecked(),
        &glib::translate::from_glib_borrow(js_result),
    )
}

fn allow_uri(uri: &str, host: &str, port: u16) -> bool {
    if let Ok(url) = url::Url::parse(uri) {
        if let (Some(h), Some(p)) = (url.host_str(), url.port()) {
            host == h && port == p
        } else {
            false
        }
    } else {
        false
    }
}

impl WebViewHelpers for WebView {
    fn respond(&self, sym: &str, id: u64, js: impl core::fmt::Display) {
        let code = format!(
            "window[Symbol.for('{}')].call({}, {});",
            sym,
            serde_json::json!(id),
            js
        );

        self.run_javascript(&code, None, |_| {});
    }

    fn only_accept_from(&self, host: &'static str, port: u16) {
        self.connect_decide_policy(move |_, decision, decision_type| match decision_type {
            PolicyDecisionType::NavigationAction => {
                let decision = decision.downcast_ref::<NavigationPolicyDecision>().unwrap();

                if let Some(uri) = decision
                    .get_navigation_action()
                    .and_then(|action| action.get_request().and_then(|request| request.get_uri()))
                {
                    if allow_uri(uri.as_str(), host, port) {
                        return true;
                    }
                }

                decision.ignore();
                true
            }
            PolicyDecisionType::Response => {
                let decision = decision.downcast_ref::<ResponsePolicyDecision>().unwrap();

                if let Some(uri) = decision
                    .get_response()
                    .and_then(|respone| respone.get_uri())
                {
                    if allow_uri(uri.as_str(), host, port) {
                        return true;
                    }
                }

                decision.ignore();
                true
            }
            _ => {
                decision.ignore();
                true
            }
        });
    }
}
