export function choose(choices) {
    const index = Math.floor(rand() * choices.length);
    return choices[index];
}

const rand_buffer = new Uint32Array(1);
export function randUint() {
    return window.crypto.getRandomValues(rand_buffer)[0];
}

export function rand() {
    return randUint() / (2 ** 32 - 1);
}