import m from "mithril";
import { stylesheet } from "typestyle";

import { fullscreen } from "/lib/styles";

const css = stylesheet({
    video: {
        position: "fixed",
        // "z-index": -100,
        ...fullscreen,
    }
});

export default {
    view: vnode => (
        <video
            class={css.video}
            src={vnode.attrs.src}
            type="video/mp4"
            loop
            autoplay
            muted
        ></video>
    )
}