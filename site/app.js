import "/lib/polyfill/webdm";

import m from "mithril";
import { forceRenderStyles } from "typestyle";

import { global as css } from "/lib/styles";
import { LoginPage } from "/lib/views";

css();

m.mount(document.body, LoginPage);
forceRenderStyles();
