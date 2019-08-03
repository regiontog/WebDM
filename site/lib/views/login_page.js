import m from "mithril";

import videos from "../../assets/backgrounds/*.mp4";
import css from '/lib/styles/login_page.scss';

import { choose } from "/lib/utils/random";
import { Stream } from "/lib/utils/rx";
import { VideoBackground, LoginForm } from "/lib/views";


export default () => {
    const video_list = Object.values(videos);
    const video = choose(video_list);
    const submit = Stream();
    const action_taken = Stream(false);
    const display_ui = Stream.merge(
        action_taken.inactive(5000).map(() => {
            m.redraw();
            return false;
        }),
        action_taken
    );

    if (!video_list.length) {
        console.error("No videos available!");
    }

    function user_action() {
        action_taken(true);
    }

    function keypress(evt) {
        user_action();

        if (evt.key === "Enter") {
            submit(evt);
        }
    }
    return {
        view: () => (
            <main class={css.main} onkeydown={keypress} onclick={user_action} >
                <VideoBackground blurred={display_ui()} cls={css.background} src={video} />
                <LoginForm show={display_ui()} submit={submit} />
            </main >
        )
    };
}