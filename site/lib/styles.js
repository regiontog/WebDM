import $if from "./macros/if.macro";

export function display_site() {
    document.body.removeAttribute("invisible");
}

export function classes(...classes) {
    let sum = "";

    for (const cls of classes) {
        if (cls) {
            sum += cls + " ";
        } else if (cls === false) {
        } else {
            throw `Invalid css class: ${cls}`;
        }
    }

    return $if(sum.length > 0).then(sum.slice(0, -1)).else(sum);
}