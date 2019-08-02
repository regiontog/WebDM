import $if from "./macros/if.macro";

export function display_site() {
    document.body.classList.remove("fade-out");
}

export function classes(...classes) {
    let sum = "";

    for (const cls of classes) {
        if (cls) {
            sum += cls + " ";
        }
    }

    return $if(sum.length > 0).then(sum.slice(0, -1)).else(sum);
}

export function style_if(cond, cls) {
    return $if(cond)
        .then(cls)
        .else(false);
}