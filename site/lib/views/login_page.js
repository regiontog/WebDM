import m from "mithril";
import stream from "mithril/stream";
import { stylesheet } from "typestyle";

import videos from "../../assets/backgrounds/*.mp4";
import { choose } from "/lib/utils/random";
import { fullscreen } from "/lib/styles";
import { HideState } from "/lib/models/hideable";
import { Hideable, VideoBackground, LoginForm } from "/lib/views";

const css = stylesheet({
    main: {
        "background-color": "black",
        cursor: "none",
        ...fullscreen
    },
});

export default vnode => {
    const video_list = Object.values(videos);
    const video = choose(video_list);
    const show = stream(false);

    if (!video_list.length) {
        console.error("No videos available!");
    }

    function keypress(evt) {
        console.log(evt);
        show(true);
    }

    return {
        view: () => (
            <main class={css.main} onkeydown={keypress}>
                <VideoBackground src={video} />
                <LoginForm show={show} />
            </main>
        )
    }
}