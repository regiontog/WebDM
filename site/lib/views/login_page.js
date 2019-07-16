import m from "mithril";
import stream from "mithril/stream";
import { stylesheet } from "typestyle";

import videos from "../../assets/backgrounds/*.mp4";
import { choose } from "/lib/utils/random";
import { maximize } from "/lib/styles";
import { VideoBackground, LoginForm } from "/lib/views";

const css = stylesheet({
    main: {
        position: "fixed",
        ...maximize
    },
    background: {
        "background-color": "black",
    }
});

export default () => {
    const video_list = Object.values(videos);
    const video = choose(video_list);
    const show = stream(false);

    if (!video_list.length) {
        console.error("No videos available!");
    }

    function user_action(evt) {
        show(true);
    }

    return {
        view: () => (
            <main class={css.main} onkeydown={user_action} onclick={user_action} >
                <VideoBackground cls={css.background} src={video} />
                <LoginForm show={show} />
            </main>
        )
    }
}