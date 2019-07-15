import { cssRule, fontFace } from "typestyle";
import { rgb } from "csx";

import montserrat from "/assets/fonts/Montserrat-Regular.ttf";

export function global() {
    cssRule('html, body', {
        padding: 0,
        margin: 0,
        ...fullscreen,
    });

    fontFace({
        fontFamily: 'Montserrat',
        src: url(montserrat),
    });
}

export function url(url) {
    return `url('${url}')`
}

export const colors = {
    white: rgb(255, 255, 255),
};

export const fullscreen = {
    height: "100%",
    width: "100%",
};