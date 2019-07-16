import m from "mithril";
import { stylesheet, classes } from "typestyle";

import { maximize } from "/lib/styles";

const css = stylesheet({
    video: {
        position: "fixed",
        "z-index": -1,
        ...maximize,
    }
});

export default {
    view: ({ attrs: { cls, ...attrs } }) => (
        <video
            {...attrs}
            class={classes(css.video, cls)}
            type="video/mp4"
            loop
            autoplay
            muted
            disablePictureInPicture
        ></video>
    )
}