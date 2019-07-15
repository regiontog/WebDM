import m from "mithril";
import { stylesheet, classes } from "typestyle";

const css = stylesheet({
    hidden: {
        opacity: 0,
    },
    visible: {
        opacity: 1,
    }
});

export default ({ attrs: { show, cls } }) => {
    return {
        view: vnode => (
            <div class={classes(cls, show() ? css.visible : css.hidden)}>
                {vnode.children}
            </div>
        )
    }
}