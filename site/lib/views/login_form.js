import m from "mithril";

import css from "/lib/styles/login_form.scss";
import $if from "../macros/if.macro";

import { classes, style_if } from "/lib/styles";
import { Stream } from "/lib/utils/rx";
import { maximize } from "/lib/styles";
import { show as show_style, max as max_style } from "/lib/styles/common.scss";


const NoUserForm = {
    view: ({ attrs: { username, password, show } }) => (
        <FormContainer cls={css.verticalFlex}>
            <input
                class={classes(css.input, style_if(show(), show_style))}
                oninput={evt => username(evt.target.value)}
                type="text"
                placeholder="username"
                autofocus
            />
            <input
                class={classes(css.input, style_if(show(), show_style))}
                oninput={evt => password(evt.target.value)}
                type="password"
                placeholder="password"
            />
        </FormContainer>
    )
};

const UsersForm = ({ attrs: { username, password, show } }) => {
    let focus = Stream();
    let active_user = Stream();

    active_user.forEach(user => {
        username(webdm.users[user].username);
    });

    active_user(0);

    return {
        view: () => (
            <FormContainer cls={css.horizontalFlex} focus={focus}>
                <button onclick={() => active_user(active_user() - 1)}>Left</button>
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
                                class={classes(css.input, style_if(show(), show_style))}
                                oninput={evt => password(evt.target.value)}
                                type="password"
                                placeholder="password"
                                autofocus
                            />
                        </li>
                    ))}
                </ul>
                <button onclick={() => active_user(active_user() + 1)}>Right</button>
            </FormContainer>
        )
    }
};

const FormContainer = ({ attrs: { focus } }) => {
    focus = focus || Stream();

    focus.forEach(elem => {
        console.debug("Setting focus: ");
        console.debug(elem);
        elem.focus();
    });

    return {
        view: ({ children, attrs: { cls } }) => (
            <div class={classes(max_style, cls)} onclick={({ target }) => {
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

export default ({ attrs: { submit, ...attrs } }) => {
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
        view: () => $if(webdm.users_hidden)
            .then(<NoUserForm {...attrs} username={username} password={password} />)
            .else(<UsersForm {...attrs} username={username} password={password} />)
    }
}