// Shared page chrome for the backback storybooks: the page menu, BACK/NEXT,
// the detail window for clickable diagram nodes, and step-through flows.
// A page sets <body data-root="../" data-page="design/why.html"> and may
// define window.MORE = {key: {title, body}} for its clickable nodes.
(function () {
  "use strict";
  var PAGES = [
    ["bb_index.html", "HOME"],
    ["design/why.html", "00 · WHY"],
    ["design/words.html", "01 · WORDS"],
    ["design/bb_bigpicture.html", "02 · BIG PICTURE"],
    ["design/components.html", "03 · COMPONENTS"],
    ["design/minimum.html", "04 · MINIMUM IDE"],
    ["runtime/bb_threads.html", "05 · PROCESSES + THREADS"],
    ["runtime/flows.html", "06 · FLOWS"],
    ["runtime/lifecycle.html", "07 · START + QUIT"],
    ["runtime/rails.html", "08 · SAFETY RAILS"]
  ];

  function menu(root, here, label) {
    var nav = document.createElement("nav");
    nav.className = "box";
    nav.setAttribute("aria-label", label);
    var row = document.createElement("div");
    row.className = "row";
    PAGES.forEach(function (p) {
      var a = document.createElement("a");
      a.href = root + p[0];
      a.textContent = p[1];
      a.className = p[0] === here ? "tag" : "tag c";
      if (p[0] === here) { a.setAttribute("aria-current", "page"); }
      row.appendChild(a);
    });
    nav.appendChild(row);
    return nav;
  }

  function pager(root, here) {
    var i = PAGES.findIndex(function (p) { return p[0] === here; });
    var nav = document.createElement("nav");
    nav.className = "pager";
    nav.setAttribute("aria-label", "Page navigation");
    var back = PAGES[i - 1], next = PAGES[i + 1];
    nav.innerHTML = (back ? '<a href="' + root + back[0] + '">&#9664; BACK</a>' : "<span></span>") +
      (next ? '<a href="' + root + next[0] + '">NEXT &#9654;</a>' : "<span></span>");
    return nav;
  }

  function chrome() {
    var body = document.body, main = document.querySelector("main.wrap");
    var root = body.dataset.root || "", here = body.dataset.page || "bb_index.html";
    var header = document.createElement("header");
    header.className = "top";
    header.innerHTML = '<a class="tag brand" href="' + root + 'bb_index.html">BACKBACK</a>';
    header.appendChild(menu(root, here, "Jump to page (top)"));
    main.insertBefore(header, main.firstChild);
    main.appendChild(document.createElement("hr"));
    main.appendChild(pager(root, here));
    main.appendChild(menu(root, here, "Jump to page (bottom)"));
  }

  function details() {
    var more = window.MORE || {};
    var dialog = document.createElement("dialog");
    dialog.className = "more";
    dialog.innerHTML = '<button class="x" aria-label="Close">&times;</button><h2></h2><div class="b"></div>';
    document.body.appendChild(dialog);
    dialog.querySelector("button.x").addEventListener("click", function () { dialog.close(); });
    dialog.addEventListener("click", function (e) { if (e.target === dialog) { dialog.close(); } });
    document.querySelectorAll("[data-more]").forEach(function (node) {
      var item = more[node.dataset.more];
      if (!item) { node.removeAttribute("data-more"); return; }
      node.setAttribute("role", "button");
      node.setAttribute("tabindex", "0");
      var open = function () {
        dialog.querySelector("h2").textContent = item.title;
        dialog.querySelector(".b").innerHTML = item.body;
        dialog.showModal();
      };
      node.addEventListener("click", open);
      node.addEventListener("keydown", function (e) {
        if (e.key === "Enter" || e.key === " ") { e.preventDefault(); open(); }
      });
    });
  }

  function flows() {
    document.querySelectorAll(".flow").forEach(function (flow) {
      var steps = flow.querySelectorAll(".rung"), at = 0;
      var ctl = document.createElement("div");
      ctl.className = "ctl";
      ctl.innerHTML = '<button data-go="-1">&#9664; STEP</button><button data-go="1">STEP &#9654;</button>' +
        '<button data-go="all">SHOW ALL</button><span class="count"></span>';
      flow.appendChild(ctl);
      var show = function () {
        steps.forEach(function (s, i) { s.classList.toggle("on", i <= at); });
        ctl.querySelector(".count").textContent = (at + 1) + " / " + steps.length;
        ctl.querySelector('[data-go="-1"]').disabled = at === 0;
        ctl.querySelector('[data-go="1"]').disabled = at === steps.length - 1;
      };
      ctl.addEventListener("click", function (e) {
        var go = e.target.dataset && e.target.dataset.go;
        if (!go) { return; }
        at = go === "all" ? steps.length - 1 : Math.min(steps.length - 1, Math.max(0, at + Number(go)));
        show();
      });
      show();
    });
  }

  document.addEventListener("DOMContentLoaded", function () { chrome(); details(); flows(); });
})();
