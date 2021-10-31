// ==UserScript==
// @name         LibMedium proxy
// @version      0.1.1
// @description  Re-writes medium.com URLs in point to libmedium
// @author       Aravinth Manivannan
// @match        https://*/*
// @match        http://*/*
// @grant        AGPLv3 or above
// ==/UserScript==

// websites to be proxied
const blacklist = ["medium.com", "blog.discord.com", "uxdesign.cc"];

// Location of the proxy
const libmediumHost = "https://libmedium.batsense.net";

(function () {
  "use strict";

  // morty has a button to go to the original site, re-writing that would be stupid
  if (!window.location.href.includes(libmediumHost)) {
    let urls = document.links;

    for (let i = 0; i < urls.length; i++) {
      blacklist.forEach((url) => {
        if (urls[i].host.includes(url)) {
          urls[i].host = libmediumHost;
        }
      });
    }
  }
})();
