window.MathJax = {
  tex: {
    inlineMath: [["\\(", "\\)"], ["$", "$"]],
    displayMath: [["\\[", "\\]"], ["$$", "$$"]],
    processEscapes: true,
    processEnvironments: true
  },
  options: {
    ignoreHtmlClass: "tex2jax_ignore",
    processHtmlClass: "tex2jax_process"
  },
  startup: {
    ready: () => {
      MathJax.startup.defaultReady();
      MathJax.typesetPromise();
    }
  }
};

if (typeof document$ !== "undefined") {
  document$.subscribe(() => {
    if (typeof MathJax !== "undefined" && MathJax.typesetPromise) {
      MathJax.typesetPromise();
    }
  });
} else {
  document.addEventListener("DOMContentLoaded", () => {
    if (typeof MathJax !== "undefined" && MathJax.typesetPromise) {
      MathJax.typesetPromise();
    }
  });
}
