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
const blacklist = [
  "medium.com",
  "blog.discord.com",
  "uxdesign.cc",
  "towardsdatascience.com",
  "hackernoon.com",
  "medium.freecodecamp.org",
  "psiloveyou.xyz",
  "betterhumans.coach.me",
  "codeburst.io",
  "theascent.pub",
  "medium.mybridge.co",
  "uxdesign.cc",
  "levelup.gitconnected.com",
  "itnext.io",
  "entrepreneurshandbook.co",
  "proandroiddev.com",
  "blog.prototypr.io",
  "thebolditalic.com",
  "blog.usejournal.com",
];

// Location of the proxy
const libmediumHost = new URL("https://libmedium.batsense.net");

const isMedium = (url) => {
  url = new URL(url);

  if (blacklist.find((bl) => url.host.includes(bl))) {
    return true;
  }

  let componenets = url.host.split(".");
  let len = componenets.length;
  if (
    len > 1 &&
    componenets[len - 1] == "com" &&
    componenets[len - 2] == "medium"
  ) {
    return true;
  }
};

(function () {
  "use strict";

  if (!window.location.href.includes(libmediumHost.host)) {
    let urls = document.links;
    for (let i = 0; i < urls.length; i++) {
      if (isMedium(urls[i])) {
        let url = urls[i];
        let path = url.pathname.split("-");
        urls[i].pathname = `/utils/post/${path[path.length - 1]}`;
        urls[i].host = libmediumHost.host;
        urls[i].search = "";
      }
    }
  }
})();
