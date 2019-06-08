(function (__rust_objects) {
    const local_webkit = window.webkit;
    delete Object.getOwnPropertySymbols;

    const CALLBACKS = Symbol.for(__rust_objects.callback_secret);

    let counter = 0;

    class Callback {
        constructor() {
            this.callbacks = {};
        }

        create(fn) {
            const id = counter++;

            this.callbacks[id] = fn;
            return id;
        }

        send(handler, message, fn) {
            const id = this.create(fn);

            handler.postMessage(JSON.stringify({
                id: id,
                data: message,
            }));
        }

        call(id, value) {
            const fn = this.callbacks[id];
            delete this.callbacks[id];
            fn(value);
        }
    }

    const callback = new Callback();

    function map_null(maybe, fn) {
        if (maybe) {
            return fn(maybe);
        } else {
            return maybe;
        }
    }

    const WebDM = {
        Greeter: class {
            constructor() {
                this.can_hibernate = __rust_objects.can_hibernate;
                this.can_restart = __rust_objects.can_restart;
                this.can_shutdown = __rust_objects.can_shutdown;
                this.can_suspend = __rust_objects.can_suspend;

                this.lock_hint = __rust_objects.lock_hint;
                this.hide_users = __rust_objects.hide_users;
                this.hostname = __rust_objects.hostname;

                this.default_session = map_null(__rust_objects.default_session, sess => new WebDM.Session(sess));

                this.sessions = __rust_objects.sessions.map(sess => new WebDM.Session(sess));
                this.users = __rust_objects.users.map(user => new WebDM.User(user));
            }

            authenticate(username, password, cancel) {
                const promise = new CancelPromise((resolve, reject) => {
                    callback.send(local_webkit.messageHandlers.auth, {
                        username: username,
                        password: password,
                    }, value => {
                        if (value) {
                            resolve();
                        } else {
                            reject();
                        }
                    });
                }, cancel);

                return promise;
            }

            exit() {
                callback.send(local_webkit.messageHandlers.exit, {}, value => { });
            }

            open_session(_session) {
                const session = _session ? {
                    name: _session.name,
                    key: _session.key,
                    comment: _session.comment
                } : __rust_objects.default_session;

                console.log(session);

                const promise = new Promise((resolve, reject) => {
                    callback.send(local_webkit.messageHandlers.open_session, session, value => {
                        if (value) {
                            resolve();
                        } else {
                            reject();
                        }
                    });
                });

                return promise;
            }

            // shutdown() { } // bool
            // hibernate() { } // bool
            // suspend() { } // bool
            // restart() { } // bool
        },
        Config: class {
            constructor() {
                this.debug = __rust_objects.debug;
                this.secure = __rust_objects.secure;
            }
        },
        Session: class {
            constructor(sess) {
                Object.assign(this, {
                    name: sess.name,
                    key: sess.key,
                    comment: sess.comment,
                });
            }
        },
        User: class {
            constructor(user) {
                Object.assign(this, {
                    display_name: user.display_name,
                    username: user.username,
                });
            }
        },
    };

    const PromiseStates = Object.freeze({
        PENDING: 0,
        FULFILLED: 1,
        REJECTED: 2,
        CANCELED: 3,
    });

    const state = new WeakMap();
    const value = new WeakMap();

    Object.assign(window, {
        webkit: undefined,
        webdm: new WebDM.Greeter(),
        greeter_config: new WebDM.Config(),
        [CALLBACKS]: callback,
        CancelPromise: class extends Promise {
            constructor(executor, cancel) {
                super((resolve, reject) => {
                    if (cancel) {
                        cancel.then(detail => {
                            this.cancel(detail);
                        });
                    }

                    executor(v => {
                        if (state[this] === PromiseStates.PENDING) {
                            state[this] = PromiseStates.FULFILLED;
                            value[this] = v;
                            resolve(v);
                        }
                    }, v => {
                        if (state[this] === PromiseStates.PENDING) {
                            state[this] = PromiseStates.REJECTED;
                            value[this] = v;
                            reject(v);
                        }
                    });
                });

                state[this] = PromiseStates.PENDING;
                this.onCancels = [];
            }

            cancel(detail) {
                if (state[this] === PromiseStates.PENDING) {
                    state[this] = PromiseStates.CANCELED;
                    value[this] = detail;
                    this.onCancels.forEach(handler => handler(detail));
                }
            }

            canceled(handler) {
                if (state[this] === PromiseStates.PENDING) {
                    this.onCancels.push(handler)
                } else if (state[this] === PromiseStates.CANCELED) {
                    handler(value[this])
                }

                return this;
            }
        }
    });
})