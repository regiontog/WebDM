import m from "mithril";

import css from "/lib/styles/video_background.scss";
import { classes, display_site } from "/lib/styles";


export default ({ attrs: { src } }) => {
    const vid = fetch(src, { mode: 'cors' }).then(response => {
        if (response.ok) {
            return response.blob();
        } else {
            console.error(response);
            throw "Response not ok";
        }
    });

    return {
        view: ({ attrs: { cls, ...attrs } }) => (
            <video
                {...attrs}
                class={classes(css.video, cls)}
                loop
                autoplay
                muted
                disablepictureinpicture
            ></video>
        ),
        oncreate({ dom }) {
            vid.then(blob => {
                console.debug(`Using loaded blob of media because of webkit video looping bug(s)?!`);

                dom.src = URL.createObjectURL(blob);
                display_site();
            }).catch(console.error);
        }
    }
}