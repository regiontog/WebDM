export function choose(choices) {
    const index = Math.floor(Math.random() * choices.length);
    return choices[index];
}