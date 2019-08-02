import fs from 'fs';

if (process.env.WEBDM_POLYFILL === "true") {
    if (!window.webdm) {
        const poly = eval(fs.readFileSync(__dirname + '../../../../src/script.js', 'utf8'));
        const webdm_data = JSON.parse(fs.readFileSync(__dirname + '../../../webdm.data.json', 'utf8'));

        console.debug("Polyfilling webdm: ", webdm_data);
        poly(webdm_data);
    } else {
        console.error("Tried to polyfill WebDM, however it is already defined!");
    }
}