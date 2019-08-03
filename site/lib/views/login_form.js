import m from "mithril";

import css from "/lib/styles/login_form.scss";
import $if from "../macros/if.macro";

import { classes } from "/lib/styles";
import { Stream } from "/lib/utils/rx";
import ArrowIcon from "/assets/icons/arrow.svg";


const NoUserForm = {
    view: ({ attrs: { username, password, show } }) => (
        <FormContainer cls={css.verticalFlex}>
            <input
                showing={show}
                class={css.input}
                oninput={evt => username(evt.target.value)}
                type="text"
                placeholder="username"
                autofocus
            />
            <input
                showing={show}
                class={css.input}
                oninput={evt => password(evt.target.value)}
                type="password"
                placeholder="password"
            />
        </FormContainer>
    )
};

const UsersForm = ({ attrs: { username, password } }) => {
    let focus = Stream();
    let active_user = Stream();

    active_user.forEach(user => {
        username(webdm.users[user].username);
    });

    active_user(0);

    return {
        view: ({ attrs: { show } }) => (
            <FormContainer cls={css.horizontalFlex} focus={focus}>
                <button disabled={active_user() === 0} class={css.button} onclick={() => active_user(active_user() - 1)}><ArrowIcon /></button>
                <ul class={css.carousel}>
                    {webdm.users.map((user, i) => (
                        <li style={{
                            transform: `translateX(${(i - active_user()) * 100}%)`,
                        }} ontransitionend={evt => {
                            if (evt.target.tagName === "LI" && evt.target.parentElement.children[active_user()] === evt.target) {
                                focus(evt.target.querySelector("input"));
                            }
                        }} class={classes(css.verticalFlex, css.carouselItem)}>
                            <span class={css.name}>{user.display_name}</span>
                            <input
                                showing={show}
                                class={css.input}
                                oninput={evt => password(evt.target.value)}
                                type="password"
                                placeholder="password"
                                autofocus
                            />
                        </li>
                    ))}
                </ul>
                <button flipped disabled={active_user() === webdm.users.length - 1} class={css.button} onclick={() => active_user(active_user() + 1)}><ArrowIcon /></button>
            </FormContainer>
        )
    }
};

const FormContainer = ({ attrs: { focus } }) => {
    focus = focus || Stream();

    focus.forEach(elem => {
        elem.focus();
    });

    return {
        view: ({ children, attrs: { cls } }) => (
            <div maximized class={cls} onclick={({ target }) => {
                if (target.tagName === "INPUT") {
                    focus(target);
                } else {
                    focus().focus();
                }
            }}>
                {children}
            </div>
        ),
        oncreate({ dom }) {
            focus(dom.querySelector("input"));
        }
    }
}

export default ({ attrs: { submit } }) => {
    const username = Stream();
    const password = Stream();

    submit.forEach(() => {
        webdm.authenticate(username(), password()).then(() => {
            webdm.open_session(webdm.sessions[0]).then(() => {
                console.log("Session started");
                webdm.exit();
            }).catch(() => {
                console.error("Could not open session");
            });
        }).catch(() => {
            console.error("Failed to log in");
        }).canceled(() => {
            console.log("Canceled log in");
        });
    });

    return {
        view: ({ attrs }) => $if(webdm.users_hidden)
            .then(<NoUserForm {...attrs} username={username} password={password} />)
            .else(<UsersForm {...attrs} username={username} password={password} />)
    }
}