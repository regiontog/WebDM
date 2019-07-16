import m from "mithril";
import { stylesheet, keyframes, classes } from "typestyle";

import { colors, fonts } from "/lib/styles";
import { Message } from "/lib/models/login_form";

import $if from "../macros/if.macro";
import { maximize } from "/lib/styles";
import { style_if } from "/lib/utils/style";

const css = stylesheet({
    form: {
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        ...maximize,
    },
    input: {
        background: "transparent",
        backgroundColor: colors.white.fade(0.3).toString(),
        color: colors.white.toString(),
        outline: "none",
        border: "1px",
        margin: "1px",
        borderRadius: "0.15rem",
        height: "2.5rem",
        fontSize: "1.5rem",
        textAlign: "center",
        opacity: 0,
        ...fonts.main_serif,
    },
    show: {
        animationName: keyframes({
            from: {
                opacity: 0,
            },
            to: {
                opacity: 1,
            }
        }),
        animationDuration: '1s',
        animationIterationCount: 1,
        animationFillMode: "both",
    },
    name: {
        color: "white",
        margin: "1px",
        fontSize: "6rem",
        userSelect: "none",
        ...fonts.main_serif,
    },
});

const NoUserForm = {
    view: ({ attrs: { show } }) => (
        <>
            <input class={classes(css.input, style_if(show(), css.show))} type="text" placeholder="username" autofocus />
            <input class={classes(css.input, style_if(show(), css.show))} type="password" placeholder="password" />
        </>
    )
};

const UsersForm = {
    view: ({ attrs: { show } }) => (
        <>
            <span class={css.name}>{webdm.users[0].display_name}</span>
            <input class={classes(css.input, style_if(show(), css.show))} type="password" placeholder="password" autofocus />
        </>
    )
};

export default ({ attrs }) => {
    let last_focused;

    return {
        view: () => (
            <div class={css.form} onfocusin={evt => last_focused = evt.target} onclick={() => last_focused.focus()} >
                {$if(webdm.users_hidden)
                    .then(<NoUserForm {...attrs} />)
                    .else(<UsersForm {...attrs} />)}
            </div>
        ),
        oncreate(vnode) {
            last_focused = vnode.dom.getElementsByTagName("input")[0];
        }
    }
}