import { cssRule, fontFace } from "typestyle";
import { rgb } from "csx";

import montserrat from "/assets/fonts/Montserrat-Regular.ttf";

export function global() {
    cssRule('html, body', {
        padding: 0,
        margin: 0,
        ...maximize,
    });

    fontFace({
        fontFamily: 'Montserrat',
        src: url(montserrat),
    });
}

export const fonts = {
    main_serif: {
        fontFamily: "Montserrat",
    }
};

export function url(url) {
    return `url('${url}')`
}

export const colors = {
    white: rgb(255, 255, 255),
    black: rgb(0, 0, 0),
};

export const maximize = {
    height: "100%",
    width: "100%",
};