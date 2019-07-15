import m from "mithril";
import { stylesheet } from "typestyle";

import { colors } from "/lib/styles";
import { Hideable } from "/lib/views";

const css = stylesheet({
    input: {
        background: "transparent",
        "background-color": colors.white.fade(0.3).toString(),
        color: colors.white.toString(),
        outline: "none",
        border: "1px",
        "border-radius": "0.15rem",
        height: "2.5rem",
        "font-family": "Montserrat",
        "font-size": "1.5rem",
        "text-align": "center",
    },
    show_transition: {
        transition: "opacity 0.8s ease-in",
    }
});

const NoUserForm = {
    view: () => (
        <>
            <Hideable cls={css.show_transition} show={show}>
                <input class={css.input} autofocus type="text" />
                <br />
                <input class={css.input} type="password" />
            </Hideable>
        </>
    )

};

const UsersForm = {

};

export default ({ attrs: { show } }) => {
    if (webdm.users_hidden) {
        return {
            view: () => (
                <Hideable cls={css.show_transition} show={show}>
                    <input class={css.input} autofocus type="text" />
                    <br />
                    <input class={css.input} type="password" />
                </Hideable>
            )
        }
    } else {
        return {
            view: () => (
                <>
                    <span>webdm.users[0]</span>
                    <Hideable cls={css.show_transition} show={show}>
                        <br />
                        <input class={css.input} autofocus type="password" />
                    </Hideable>
                </>
            )
        }
    }
}