import m from "mithril";

import "/lib/styles/global.scss";
import "/lib/polyfill/webdm";
import { LoginPage } from "/lib/views";

m.mount(document.body, LoginPage);